// `xs` is a native tool: it runs cargo, binds an http socket and hands out
// websocket frames. Building it for wasm cannot work, say so once and clearly
// rather than through a pile of unresolved imports.
#[cfg(target_arch = "wasm32")]
compile_error!(
    "xs-cli is a native tool, it cannot be built for wasm32. \
     Build it for the host: cargo build -p xs-cli --target <host triple> \
     (or drop [build] target from the project's .cargo/config.toml, which the \
     tool does not need: it passes --target itself)."
);

#[cfg(not(target_arch = "wasm32"))]
mod build;
#[cfg(not(target_arch = "wasm32"))]
mod cargo_metadata;
#[cfg(not(target_arch = "wasm32"))]
mod handle_static;
#[cfg(not(target_arch = "wasm32"))]
mod html;
#[cfg(not(target_arch = "wasm32"))]
mod options;
#[cfg(not(target_arch = "wasm32"))]
mod serve;
#[cfg(not(target_arch = "wasm32"))]
mod watcher;

#[cfg(not(target_arch = "wasm32"))]
use options::{Command, Options, USAGE};

/// Never reached on wasm, the compile_error above stops the build first.
#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
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
        Command::Serve => serve::run(&options),
        Command::Build => build::run(&options),
    }
}

/// The package to build, telling which one was picked when the caller left it
/// to the cargo metadata of the project.
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_package(options: &Options) -> String {
    match options.resolve_package() {
        Ok(package) => package,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
