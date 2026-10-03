use std::io::BufRead;
use std::sync::mpsc::Sender;

use crate::options::Options;

use nanoserde::DeJson;

#[derive(Debug, DeJson)]
struct CargoBuildMessageSpan {
    file_name: String,
}

impl Default for CargoBuildMessageSpan {
    fn default() -> Self {
        Self {
            file_name: String::new(),
        }
    }
}

#[derive(Debug, DeJson)]
struct CargoBuildMessage {
    rendered: String,
    spans: Vec<CargoBuildMessageSpan>,
}

impl Default for CargoBuildMessage {
    fn default() -> Self {
        Self {
            rendered: String::new(),
            spans: Vec::new(),
        }
    }
}

#[derive(Debug, DeJson)]
struct CargoBuildOutput {
    reason: String,
    executable: Option<String>,
    message: Option<CargoBuildMessage>,
}

impl Default for CargoBuildOutput {
    fn default() -> Self {
        Self {
            reason: String::new(),
            executable: None,
            message: None,
        }
    }
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
    let args: Vec<String> = vec![
        "watch".to_string(),
        "-d".to_string(),
        "0.1".to_string(),
        "-x".to_string(),
        format!("{} --message-format=json", options.cargo_args().join(" ")),
    ];
    let env = options.cargo_env();
    let wasm_name = wasm_name.to_string();
    std::thread::spawn(move || build(args, env, wasm_name, sender));
    receiver
}

fn build(
    args: Vec<String>,
    env: Vec<(&'static str, String)>,
    wasm_name: String,
    sender: Sender<Rebuild>,
) {
    let mut started = std::time::Instant::now();
    let mut version = 0;
    let mut command = std::process::Command::new("cargo");
    command
        .args(args.clone())
        .envs(env)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            eprintln!("failed to run cargo: {error}");
            return;
        }
    };
    let stdout = child.stdout.take().expect("stdout was piped");
    let reader = std::io::BufReader::new(stdout);
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
                if let Some(message) = output.message {
                    if let Some(span) = message.spans.first() {
                        println!("{}: {}", span.file_name, message.rendered);
                    }
                };
            }
            _ => {}
        }
    }
    let _ = child.wait();
}
