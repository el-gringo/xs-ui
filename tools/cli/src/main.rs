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

pub fn resolve_package(options: &Options) -> String {
    match options.resolve_package() {
        Ok(package) => package,
        Err(others) => {
            eprintln!("error: nothing to build, pass a package or an --example");
            for other in others {
                eprintln!("  candidate: {other}");
            }
            std::process::exit(1);
        }
    }
}
