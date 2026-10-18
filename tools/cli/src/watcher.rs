use std::io::BufRead;
use std::os::unix::process::CommandExt;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::Sender;

use crate::options::Options;

use nanoserde::DeJson;

/// The process group of the `cargo watch` this server started, or 0 while there is
/// none. Written by the thread that spawned it and read by the signal handler, so
/// it is the one piece of state that outlives every lock here.
static BUILD_PID: AtomicI32 = AtomicI32::new(0);

#[derive(Debug, Default, DeJson)]
struct CargoBuildMessageSpan {
    file_name: String,
}

#[derive(Debug, Default, DeJson)]
struct CargoBuildMessage {
    rendered: String,
    spans: Vec<CargoBuildMessageSpan>,
}

#[derive(Debug, Default, DeJson)]
struct CargoBuildOutput {
    reason: String,
    executable: Option<String>,
    message: Option<CargoBuildMessage>,
}

/// What the browser is told when the app is rebuilt: the url to load, with a
/// fresh query so the wasm is not served from the browser cache.
#[allow(dead_code)]
pub struct Rebuild {
    pub url: String,
    pub path: String,
    pub duration_ms: u128,
}

/// Runs `cargo watch`, rebuilding the target and reporting every artifact.
///
/// The build is left exactly as the project configured it: no rustflags, no
/// build-std, those come from its own cargo configuration, unless `--legacy`
/// asks for the old-webview exception.
pub fn watch(options: &Options, wasm_name: &str) -> std::sync::mpsc::Receiver<Rebuild> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let Some(mut child) = spawn_watch(options) else {
        return receiver;
    };
    let group = child.id() as i32;
    BUILD_PID.store(group, Ordering::SeqCst);
    let stdout = child.stdout.take().expect("stdout was piped");
    let wasm_name = wasm_name.to_string();
    std::thread::spawn(move || {
        read_output(std::io::BufReader::new(stdout), wasm_name, sender);
        let _ = child.wait();
        // The build ended on its own - cargo watch exiting is not supposed to
        // happen, but a toolchain that is missing is one way to get here - so
        // there is nothing left for the signal handler to take down.
        let _ = BUILD_PID.compare_exchange(group, 0, Ordering::SeqCst, Ordering::SeqCst);
    });
    receiver
}

/// Starts `cargo watch`, in a process group of its own.
///
/// The group is what makes stopping the server stop the build: see
/// [`terminate_on_signal`]. Without it the watcher is a child in whatever group
/// started the server, and taking that group down does reach it - which is why a
/// Ctrl-C in a terminal has always worked and why nothing about this looked like a
/// problem until something signalled the server alone.
fn spawn_watch(options: &Options) -> Option<std::process::Child> {
    let args: Vec<String> = vec![
        "watch".to_string(),
        "-d".to_string(),
        "0.1".to_string(),
        "-x".to_string(),
        format!("{} --message-format=json", options.cargo_args().join(" ")),
    ];
    let mut command = std::process::Command::new("cargo");
    command
        .args(args)
        .envs(options.cargo_env())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .process_group(0);
    match command.spawn() {
        Ok(child) => Some(child),
        Err(error) => {
            eprintln!("failed to run cargo: {error}");
            None
        }
    }
}

/// Take the build down whenever this process is told to stop.
///
/// A `cargo watch` that outlives its server is the expensive kind of leak here, and
/// the reason it is easy to miss: the watcher is a child, so Ctrl-C in a terminal
/// reaches it too and everything looks correct. Anything that signals *this
/// process* does not - a supervisor, a script, a `nohup`, a panic - and the watcher
/// is then reparented and goes on doing the only thing it exists to do, rebuilding
/// on every edit. A second `xs serve` starts a second watcher on the same target
/// directory, and from there the memory is rustc's: one per watcher, rebuilt over
/// and over.
///
/// The child is its own process group, so the handler kills that group rather than
/// one pid. Killing `cargo watch` alone would leave the `cargo` and `rustc` it was
/// running at the time, which is the expensive part.
///
/// This installs handlers rather than waiting to be dropped on the way out because
/// a signal does not unwind: there is no `Drop` to run for `SIGTERM`, and the
/// process that leaked the watcher is the one that is being told to leave.
pub fn terminate_on_signal() {
    const SIGHUP: i32 = 1;
    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;
    const SIGKILL: i32 = 9;

    unsafe extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
        fn kill(pid: i32, sig: i32) -> i32;
        fn _exit(code: i32) -> !;
    }

    /// Async-signal-safe by construction: an atomic load and `kill`, which is all
    /// the list allows. Nothing here allocates, locks or logs.
    extern "C" fn on_terminate(_signum: i32) {
        let group = BUILD_PID.load(Ordering::SeqCst);
        if group > 0 {
            // Negative: the whole group, which the child's own pid names because
            // `process_group(0)` made it the leader of one.
            unsafe { kill(-group, SIGKILL) };
        }
        unsafe { _exit(1) };
    }

    let handler = on_terminate as extern "C" fn(i32) as usize;
    for signum in [SIGHUP, SIGINT, SIGTERM] {
        unsafe { signal(signum, handler) };
    }
}

/// Reads cargo's json output, reporting every artifact it finishes with.
///
/// Takes nothing from [`Options`]: the flags and the environment were already read
/// to spawn the build, and nothing else about the options bears on reading cargo's
/// output.
fn read_output(reader: impl BufRead, wasm_name: String, sender: Sender<Rebuild>) {
    let mut started = std::time::Instant::now();
    let mut version = 0;
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => continue,
        };
        let output: CargoBuildOutput = match DeJson::deserialize_json(&line) {
            Ok(output) => output,
            Err(_) => {
                if line.starts_with("[Running") {
                    started = std::time::Instant::now();
                }
                CargoBuildOutput::default()
            }
        };
        match output.reason.as_str() {
            "compiler-artifact" => {
                if let Some(executable) = output.executable {
                    version += 1;
                    let url = format!("{wasm_name}?v={version}");
                    println!("Updated {url} in {}ms", started.elapsed().as_millis());
                    // A dropped receiver only means nobody is watching.
                    let _ = sender.send(Rebuild {
                        url,
                        path: executable,
                        duration_ms: started.elapsed().as_millis(),
                    });
                }
            }
            "compiler-message" => {
                if let Some(message) = output.message
                    && let Some(span) = message.spans.first()
                {
                    println!("{}: {}", span.file_name, message.rendered);
                };
            }
            _ => {}
        }
    }
}
