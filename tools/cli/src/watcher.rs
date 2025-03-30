use nanoserde::DeJson;
use std::{io::BufRead, sync::mpsc::Sender};

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
struct CargoBuildMessageChild {
    //children: Vec<CargoBuildMessageChild>,
    //code: None,
    //level: String,
    //message: String,
    //rendered: Option<String>,
    //spans: Vec<CargoBuildMessageSpan>
}

#[derive(Debug, DeJson)]
struct CargoBuildMessage {
    rendered: String,
    //children: Vec<CargoBuildMessageChild>,
    spans: Vec<CargoBuildMessageSpan>,
}

impl Default for CargoBuildMessage {
    fn default() -> Self {
        Self {
            rendered: String::new(),
            //children: Vec::new(),
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

fn build(target_directory: String, args: Vec<String>, sender: Sender<String>) {
    let start = target_directory.len();
    let command = std::process::Command::new("cargo")
        .env("RUSTFLAGS", "-Ctarget-feature=-reference-types")
        .args(args)
        .stdout(std::process::Stdio::piped())
        //.stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn cargo watch");

    let stdout = command.stdout.unwrap();
    let reader = std::io::BufReader::new(stdout);
    let stdout_lines = reader.lines();
    let mut before = std::time::Instant::now();
    for line in stdout_lines {
        let line = line.unwrap();
        //  println!("line: {}", line);
        let output: CargoBuildOutput = match DeJson::deserialize_json(&line) {
            Ok(output) => output,
            Err(_) => {
                if line.starts_with("[Running") {
                    before = std::time::Instant::now();
                }
                CargoBuildOutput::default()
            }
        };
        match output.reason.as_str() {
            "compiler-artifact" => {
                if let Some(executable) = output.executable {
                    let wasm_pathname: String = executable
                        .chars()
                        .skip(start)
                        .take(executable.len() - start)
                        .collect();
                    println!(
                        "Updated: {} in {}ms",
                        wasm_pathname,
                        before.elapsed().as_millis()
                    );
                    sender.send(wasm_pathname).unwrap();
                }
            }
            "compiler-message" => {
                if let Some(message) = output.message {
                    let spans = message.spans;
                    if let Some(span) = spans.first() {
                        println!("file_name: {}", span.file_name);
                    }
                    println!("{}", message.rendered);
                };
            }
            "build-finished" => {}
            _ => {}
        }
    }
}
/*
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
pub fn watch_notify(target_directory: String, additional_args: Vec<String>, sender: Sender<String>) {
    let path = source_path.clone();
    let watch_path = Path::new(path.as_str());
    println!("source_path: {:?}", source_path);

    let mut args: Vec<String> = vec![
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "--message-format=json",
    ]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    args.extend(additional_args);

    build(
        source_path.clone(),
        args.clone(),
        sender.clone()
    );

    let watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        let target_directory = target_directory.clone();
        let args = args.clone();
        let sender = sender.clone();
        match res {
            Ok(event) => {
                println!("event.kind: {:?}", event.kind);
                let update = match event.kind {
                    EventKind::Create(_) => true,
                    EventKind::Modify(_) => true,
                    _ => false
                };
                if update {
                    //build(target_directory, args, sender);
                }
            }
            Err(e) => println!("Error {:?}", e),
        }
    });
    if let Ok(mut watcher) = watcher {
        let _ = watcher.watch(watch_path, RecursiveMode::Recursive);
    };
}
*/

pub fn watch(target_directory: String, additional_args: Vec<String>, sender: Sender<String>) {
    let mut args: Vec<String> = vec![
        "-Zbuild-std=panic_abort,std",
        "watch",
        "-d",
        "0.1",
        "-x",
        "build --target wasm32-unknown-unknown --message-format=json"
    ]
        .iter()
        .map(|arg| arg.to_string())
        .collect();
    args.extend(additional_args);
    std::thread::spawn(|| {
        build(target_directory, args, sender);
    });
}
