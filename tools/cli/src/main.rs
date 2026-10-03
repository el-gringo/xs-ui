// `xs` is a native tool: it runs cargo, binds an http socket and hands out
// websocket frames. It is never built for wasm32, so nothing here is gated on
// the host.
mod build;
mod cargo_metadata;
mod handle_static;
mod html;
mod options;
mod serve;
mod watcher;

use options::{Command, Options, USAGE};

pub fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("{USAGE}");
        return;
    }
    let options = match options::parse(&args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("error: {error}\n\n{USAGE}");
            std::process::exit(1);
        }
    };

    match options.command {
        Command::Help => println!("{USAGE}"),
        Command::Serve => serve::run(&options),
        Command::Run | Command::Build => build::run(&options),
    }
}

/// The package to build, telling which one was picked when the caller left it
/// to the cargo metadata of the project.
pub fn resolve_package(options: &Options) -> String {
    match options.resolve_package() {
        Ok(package) => package,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
