use std::path::PathBuf;

/// The page served when the caller names none: the one the binding system
/// ships, which any xs-ui project can serve as it is.
const SHARED_HTML: &str = "crates/xs-js-sys/html/index.html";
/// Used when the project has no xs-ui page to share.
const LOCAL_HTML: &str = "index.html";
const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_DIST: &str = "dist";
const DEFAULT_TARGET: &str = "wasm32-unknown-unknown";

pub const USAGE: &str = "\
xs - dev server and packager for wasm projects

usage:
  xs serve [package] [html] [options]    serve the project, rebuilding it on change
  xs build [package] [html] [options]    build the project into --dist

  The wasm loader is injected where the html has a <link rel=\"rust\" />, or at
  the end of its body when it has none. In dev the injected script also connects
  to the dev server and reloads the page on every rebuild.

  How the wasm is built is left to the project: xs runs `cargo` in it, so
  .cargo/config.toml holds the build-std and the rustflags.

arguments:
  [package]            package to build, its default binary when omitted
  [html]               html file to serve and inject into
                       (default: crates/xs-js-sys/html/index.html, else ./index.html)

options:
  -p, --package <name>    same as the [package] argument
  -e, --example <name>    build an example of the package instead of its binary
      --target <triple>   target triple to build for (default: wasm32-unknown-unknown)
      --host <addr>       address to serve on (default: 0.0.0.0)
      --port <port>       port of the dev server (default: 8080)
      --dist <dir>        output directory of `build` (default: dist)
      --release           build with the release profile (default in `build`)
      --debug             build with the dev profile
      --no-watch          do not rebuild on change, `serve` only
  -h, --help              print this help
";

#[derive(Debug, PartialEq)]
pub enum Command {
    Serve,
    Build,
}

#[derive(Debug)]
pub struct Options {
    pub command: Command,
    /// None until it is resolved from the cargo metadata.
    pub package: Option<String>,
    /// `true` when the wasm comes from an example target instead of the binary.
    pub example: Option<String>,
    pub html: PathBuf,
    pub target: String,
    pub host: String,
    pub port: u16,
    pub dist: PathBuf,
    pub release: bool,
    pub watch: bool,
}

impl Options {
    /// The package to build: the one given on the command line, or the default
    /// binary crate of the project.
    pub fn resolve_package(&self) -> Result<String, Vec<String>> {
        match &self.package {
            Some(package) => Ok(package.clone()),
            None => {
                let mut candidates = crate::cargo_metadata::default_bins();
                match candidates.len() {
                    0 => Err(Vec::new()),
                    _ => Ok(candidates.remove(0)),
                }
            }
        }
    }

    /// `build` defaults to --release, `serve` to the dev profile.
    pub fn profile(&self) -> &'static str {
        if self.release { "release" } else { "debug" }
    }

    /// Where cargo puts the wasm of this target.
    pub fn wasm_path(&self, target_directory: &str) -> PathBuf {
        let mut path = PathBuf::from(target_directory)
            .join(&self.target)
            .join(self.profile());
        match &self.example {
            Some(example) => {
                path.push("examples");
                path.push(format!("{example}.wasm"));
            }
            None => {
                let package = self.resolve_package().unwrap_or_default();
                path.push(format!("{package}.wasm"));
            }
        }
        path
    }

    /// cargo arguments to build this target, `--message-format=json` excluded.
    /// Everything else (target dir, rustflags, build-std) comes from the
    /// project's own cargo configuration.
    pub fn cargo_args(&self) -> Vec<String> {
        let package = self.resolve_package().unwrap_or_default();
        let mut args = vec!["build".to_string(), "--package".to_string(), package];
        if let Some(example) = &self.example {
            args.push("--example".to_string());
            args.push(example.clone());
        }
        args.push("--target".to_string());
        args.push(self.target.clone());
        if self.release {
            args.push("--release".to_string());
        }
        args
    }
}

/// The shared page when the project has one, its own `index.html` otherwise.
pub fn default_html() -> PathBuf {
    let shared = PathBuf::from(SHARED_HTML);
    if shared.is_file() {
        shared
    } else {
        PathBuf::from(LOCAL_HTML)
    }
}

pub fn parse(args: &[String]) -> Result<Options, String> {
    let mut command = None;
    let mut package = None;
    let mut example = None;
    let mut html = None;
    let mut target = None;
    let mut host = None;
    let mut port = None;
    let mut dist = None;
    let mut release = None;
    let mut watch = true;

    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        // Every option taking a value reads it from the next argument.
        let mut value = || {
            index += 1;
            args.get(index)
                .cloned()
                .ok_or_else(|| format!("{arg} needs a value"))
        };
        match arg {
            "serve" => command = Some(Command::Serve),
            "build" => command = Some(Command::Build),
            "-p" | "--package" => package = Some(value()?),
            "-e" | "--example" => example = Some(value()?),
            "--target" => target = Some(value()?),
            "--host" => host = Some(value()?),
            "--port" => {
                let raw = value()?;
                port = Some(
                    raw.parse()
                        .map_err(|_| format!("--port expects a number, got {raw}"))?,
                );
            }
            "--dist" => dist = Some(PathBuf::from(value()?)),
            "--release" => release = Some(true),
            "--debug" => release = Some(false),
            "--no-watch" => watch = false,
            // Free standing arguments are the package then the html, so both
            // `xs serve counter` and `xs counter` work. Anything starting with
            // a dash is a mistyped option, not a positional.
            other if other.starts_with('-') => {
                return Err(format!("unknown option: {other}"));
            }
            other if package.is_none() => package = Some(other.to_string()),
            other if html.is_none() => html = Some(PathBuf::from(other)),
            other => return Err(format!("unexpected argument: {other}")),
        }
        index += 1;
    }

    let command = match command {
        // `xs peertube` stays valid, it means `serve peertube`.
        None => Command::Serve,
        Some(command) => command,
    };
    let release = release.unwrap_or(command == Command::Build);

    Ok(Options {
        command,
        package,
        example,
        html: html.unwrap_or_else(default_html),
        target: target.unwrap_or_else(|| DEFAULT_TARGET.to_string()),
        host: host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
        port: port.unwrap_or(DEFAULT_PORT),
        dist: dist.unwrap_or_else(|| PathBuf::from(DEFAULT_DIST)),
        release,
        watch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Options, String> {
        parse(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn defaults_to_serving_in_dev() {
        let options = parse_args(&[]).unwrap();
        assert_eq!(options.command, Command::Serve);
        // No package: the cargo metadata decides which binary to serve.
        assert_eq!(options.package, None);
        assert_eq!(options.html, default_html());
        assert_eq!(options.target, "wasm32-unknown-unknown");
        assert_eq!(options.host, "0.0.0.0");
        assert_eq!(options.port, 8080);
        assert!(!options.release);
        assert!(options.watch);
    }

    #[test]
    fn a_bare_package_is_a_serve_target() {
        let options = parse_args(&["counter"]).unwrap();
        assert_eq!(options.command, Command::Serve);
        assert_eq!(options.package.as_deref(), Some("counter"));
    }

    #[test]
    fn the_shared_page_is_the_default_html() {
        let shared = PathBuf::from(SHARED_HTML);
        if shared.is_file() {
            assert_eq!(default_html(), shared);
        } else {
            assert_eq!(default_html(), PathBuf::from(LOCAL_HTML));
        }
    }

    #[test]
    fn the_html_is_the_second_positional() {
        let options = parse_args(&["serve", "counter", "pages/index.html"]).unwrap();
        assert_eq!(options.package.as_deref(), Some("counter"));
        assert_eq!(options.html, PathBuf::from("pages/index.html"));
    }

    #[test]
    fn the_package_flag_also_sets_the_package() {
        let options = parse_args(&["serve", "-p", "playground"]).unwrap();
        assert_eq!(options.package.as_deref(), Some("playground"));
    }

    #[test]
    fn a_third_positional_is_rejected() {
        assert!(parse_args(&["serve", "counter", "a.html", "b.html"]).is_err());
    }

    #[test]
    fn host_port_and_target_can_be_set() {
        let options = parse_args(&[
            "serve",
            "--host",
            "127.0.0.1",
            "--port",
            "9000",
            "--target",
            "wasm32-wasi",
        ])
        .unwrap();
        assert_eq!(options.host, "127.0.0.1");
        assert_eq!(options.port, 9000);
        assert_eq!(options.target, "wasm32-wasi");
    }

    #[test]
    fn build_defaults_to_release() {
        let options = parse_args(&["build", "peertube"]).unwrap();
        assert_eq!(options.command, Command::Build);
        assert!(options.release);
        assert_eq!(options.profile(), "release");
    }

    #[test]
    fn wasm_path_follows_the_target_kind() {
        let mut options = parse_args(&["serve", "peertube"]).unwrap();
        let _ = &options.package;
        assert_eq!(
            options.wasm_path("/t"),
            PathBuf::from("/t/wasm32-unknown-unknown/debug/peertube.wasm")
        );

        options = parse_args(&["serve", "-p", "xs-js-sys", "-e", "webgl", "--release"]).unwrap();
        assert_eq!(
            options.wasm_path("/t"),
            PathBuf::from("/t/wasm32-unknown-unknown/release/examples/webgl.wasm")
        );
        assert!(options.cargo_args().contains(&"webgl".to_string()));
    }

    #[test]
    fn the_build_leaves_the_rustflags_to_the_project() {
        let options = parse_args(&["build", "app"]).unwrap();
        let args = options.cargo_args();
        assert!(args.starts_with(&["build".to_string(), "--package".to_string()]));
        // No -Zbuild-std, no RUSTFLAGS: the project config carries them.
        assert!(!args.iter().any(|arg| arg.contains("build-std")));
        assert!(!args.iter().any(|arg| arg.contains("RUSTFLAGS")));
    }

    #[test]
    fn rejects_unknown_arguments() {
        assert!(parse_args(&["--nope"]).is_err());
        assert!(parse_args(&["--port"]).is_err());
        assert!(parse_args(&["--port", "x"]).is_err());
        // --html is gone, the html is positional now.
        assert!(parse_args(&["--html", "a.html"]).is_err());
    }
}
