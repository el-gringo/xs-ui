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

/// `-Zbuild-std` for a legacy build. Rebuilding `std` from source is what
/// keeps post-MVP instructions out of the module: `-Ctarget-cpu=mvp` only shapes
/// the code this workspace compiles, and the precompiled `std` rustup ships is
/// built *with* sign-extension and bulk-memory, which Chrome 63 rejects with
/// "Invalid opcode".
pub const BUILD_STD: &str = "-Zbuild-std=panic_abort,std";
/// The codegen half of the same requirement. Passed through `RUSTFLAGS` because
/// that is the only channel available to a tool that has no `.cargo/config.toml`
/// of its own, and an environment variable replaces the project `rustflags`
/// outright, so this has to carry both halves to stand on its own.
pub const LEGACY_RUSTFLAGS: &str = "-Ctarget-cpu=mvp -Ctarget-feature=-sign-ext";

pub const USAGE: &str = "\
xs - build, run and serve xs-ui projects

usage:
  xs                                     print this help
  xs run [package] [options]             build and run the project (native)
  xs build [package] [options]           build the project
  xs serve [package] [html] [options]    serve the web build, rebuilding on change

  `run` and `build` target the host by default; pass --platform web for the wasm.
  `serve` always serves the web build, whatever --platform says.

  The wasm loader is injected where the html has a <link rel=\"rust\" />, or at
  the end of its body when it has none. In dev the injected script also connects
  to the dev server and reloads the page on every rebuild.

  xs runs `cargo` in the project and passes only --target and --release, so the
  project's own cargo configuration decides the rest. Pass --legacy for the one
  thing that has to be right on an old webview, see below.

arguments:
  [package]            package to build, its default binary when omitted
  [html]               html file to serve and inject into, `serve` only
                       (default: crates/xs-js-sys/html/index.html, else ./index.html)

options:
  -p, --package <name>    same as the [package] argument
  -e, --example <name>    build an example of the package instead of its binary
      --platform <name>   `native` for the host, `web` for wasm32
                          (default: native, always web for `serve`)
      --target <triple>   build for this target instead of the platform default
      --legacy            web only: MVP codegen and a rebuilt std, so the module
                          runs on browsers that predate sign-extension and
                          bulk-memory (Chrome 74 and 75). Off by default,
                          because rebuilding std costs about 15 seconds of cold
                          build
      --host <addr>       address to serve on (default: 0.0.0.0)
      --port <port>       port of the dev server (default: 8080)
      --dist <dir>        output directory of `build --platform web` (default: dist)
      --release           build with the release profile (default in `build`)
      --debug             build with the dev profile
      --no-watch          do not rebuild on change, `serve` only
  -h, --help              print this help
";

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// No arguments at all: print the usage and change nothing.
    Help,
    Serve,
    Build,
    Run,
}

/// What a build targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    /// The host, built by cargo with no `--target`.
    Native,
    /// `wasm32-unknown-unknown`, for a browser.
    Web,
}

impl Platform {
    /// Parses the `--platform` value.
    fn parse(value: &str) -> Result<Platform, String> {
        match value {
            "native" => Ok(Platform::Native),
            "web" => Ok(Platform::Web),
            other => Err(format!("--platform expects native or web, got {other}")),
        }
    }
}

#[derive(Debug)]
pub struct Options {
    pub command: Command,
    /// None until it is resolved from the cargo metadata.
    pub package: Option<String>,
    /// `true` when the wasm comes from an example target instead of the binary.
    pub example: Option<String>,
    pub html: PathBuf,
    pub platform: Platform,
    /// `None` means the host, which is what cargo wants for a native build.
    pub target: Option<String>,
    pub host: String,
    pub port: u16,
    pub dist: PathBuf,
    pub release: bool,
    pub watch: bool,
    /// Build for an old webview: MVP codegen and a rebuilt std.
    pub legacy: bool,
}

impl Options {
    /// The no-argument invocation: print the usage.
    fn help() -> Options {
        Options {
            command: Command::Help,
            package: None,
            example: None,
            html: default_html(),
            platform: Platform::Native,
            target: None,
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            dist: PathBuf::from(DEFAULT_DIST),
            release: false,
            watch: true,
            legacy: false,
        }
    }

    /// Whether this invocation produces something a browser loads.
    pub fn is_web(&self) -> bool {
        self.platform == Platform::Web
    }

    /// The package to build: the one given on the command line, the one that
    /// owns the requested example, or the project's default binary crate.
    ///
    /// The error is ready to print, it names the candidates when the target is
    /// not there.
    pub fn resolve_package(&self) -> Result<String, String> {
        if let Some(package) = &self.package {
            return Ok(package.clone());
        }
        if let Some(example) = &self.example {
            return match crate::cargo_metadata::packages_with_example(example)
                .into_iter()
                .next()
            {
                Some(package) => Ok(package),
                None => Err(missing_example(example)),
            };
        }
        match crate::cargo_metadata::default_bins().into_iter().next() {
            Some(package) => Ok(package),
            None => Err("no binary target in this workspace, pass a package".to_string()),
        }
    }

    /// `build` defaults to --release, `serve` to the dev profile.
    pub fn profile(&self) -> &'static str {
        if self.release { "release" } else { "debug" }
    }

    /// Where cargo puts the wasm of this target. Only `serve` and a web build
    /// have one; a native build produces a binary cargo names after the package.
    pub fn wasm_path(&self, target_directory: &str) -> PathBuf {
        let mut path = PathBuf::from(target_directory)
            .join(self.target.as_deref().unwrap_or(DEFAULT_TARGET))
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

    /// cargo arguments for this invocation, `--message-format=json` excluded.
    ///
    /// A native build gets no `--target` at all, which is how cargo is told to
    /// use the host. Everything else (target dir, rustflags, build-std) comes
    /// from the project's own cargo configuration, unless `--legacy` asks for
    /// the one exception this tool knows about.
    pub fn cargo_args(&self) -> Vec<String> {
        let package = self.resolve_package().unwrap_or_default();
        let verb = if self.command == Command::Run {
            "run"
        } else {
            "build"
        };
        let mut args = vec![verb.to_string(), "--package".to_string(), package];
        if let Some(example) = &self.example {
            args.push("--example".to_string());
            args.push(example.clone());
        }
        if let Some(target) = &self.target {
            args.push("--target".to_string());
            args.push(target.clone());
        }
        if self.release {
            args.push("--release".to_string());
        }
        if self.legacy {
            args.push(BUILD_STD.to_string());
        }
        args
    }

    /// Environment overrides for that cargo invocation.
    ///
    /// Only `--legacy` has one, and it is empty otherwise so that cargo keeps
    /// using whatever `RUSTFLAGS` the developer exported.
    pub fn cargo_env(&self) -> Vec<(&'static str, String)> {
        if self.legacy {
            vec![("RUSTFLAGS", LEGACY_RUSTFLAGS.to_string())]
        } else {
            Vec::new()
        }
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

/// The html is the only argument that is a path, so its extension is enough to
/// tell the two positionals apart.
fn is_html(arg: &str) -> bool {
    let arg = arg.to_ascii_lowercase();
    arg.ends_with(".html") || arg.ends_with(".htm")
}

fn missing_example(example: &str) -> String {
    let mut error = format!("no example target named `{example}` in this workspace");
    let available = crate::cargo_metadata::examples();
    if available.is_empty() {
        return error;
    }
    error.push_str(", it has:");
    for (package, names) in available {
        error.push_str(&format!("\n  {package}: {}", names.join(", ")));
    }
    error
}

pub fn parse(args: &[String]) -> Result<Options, String> {
    // Bare `xs` explains itself rather than starting a server nobody asked for.
    if args.is_empty() {
        return Ok(Options::help());
    }
    let mut command = None;
    let mut package = None;
    let mut example = None;
    let mut html = None;
    let mut platform = None;
    let mut target = None;
    let mut host = None;
    let mut port = None;
    let mut dist = None;
    let mut release = None;
    let mut watch = true;
    let mut legacy = false;

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
            "run" => command = Some(Command::Run),
            "-p" | "--package" => package = Some(value()?),
            "-e" | "--example" => example = Some(value()?),
            "--platform" => platform = Some(Platform::parse(&value()?)?),
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
            "--legacy" => legacy = true,
            // Free standing arguments are the package then the html, so both
            // `xs serve counter` and `xs counter` work. Anything starting with
            // a dash is a mistyped option, not a positional, and an html file
            // is always the html even when it comes first: `xs serve -e webgl
            // app/index.html` should not read the path as the package.
            other if other.starts_with('-') => {
                return Err(format!("unknown option: {other}"));
            }
            other if is_html(other) && html.is_none() => html = Some(PathBuf::from(other)),
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

    // `serve` exists to hand a browser the wasm, so it is the web build whatever
    // was asked for. Saying so beats honouring a flag the action cannot honour.
    let platform = match (command.clone(), platform) {
        (Command::Serve, _) => Platform::Web,
        (_, Some(platform)) => platform,
        (_, None) => Platform::Native,
    };
    if legacy && platform != Platform::Web {
        return Err("--legacy is a wasm option, pass --platform web".to_string());
    }
    if command == Command::Run && platform != Platform::Native {
        return Err("run executes the binary, which the web build cannot do; \
             use `serve` to preview the web build"
            .to_string());
    }
    // An explicit --target wins; otherwise the platform picks. Native leaves it
    // unset so cargo uses the host.
    let target = match target {
        Some(target) => Some(target),
        None if platform == Platform::Web => Some(DEFAULT_TARGET.to_string()),
        None => None,
    };

    Ok(Options {
        command,
        package,
        example,
        html: html.unwrap_or_else(default_html),
        platform,
        target,
        host: host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
        port: port.unwrap_or(DEFAULT_PORT),
        dist: dist.unwrap_or_else(|| PathBuf::from(DEFAULT_DIST)),
        release,
        watch,
        legacy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Options, String> {
        parse(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn bare_invocation_prints_the_help() {
        // Nothing to do is not a reason to start a server.
        let options = parse_args(&[]).unwrap();
        assert_eq!(options.command, Command::Help);
    }

    #[test]
    fn a_bare_package_still_means_serve() {
        let options = parse_args(&["counter"]).unwrap();
        assert_eq!(options.command, Command::Serve);
    }

    #[test]
    fn serve_serves_the_web_build() {
        let options = parse_args(&["serve"]).unwrap();
        assert_eq!(options.command, Command::Serve);
        assert!(options.is_web());
        // No package: the cargo metadata decides which binary to serve.
        assert_eq!(options.package, None);
        assert_eq!(options.target.as_deref(), Some("wasm32-unknown-unknown"));
        assert_eq!(options.html, default_html());
        assert_eq!(options.host, "0.0.0.0");
        assert_eq!(options.port, 8080);
        assert!(!options.release);
        assert!(options.watch);
    }

    #[test]
    fn serve_ignores_a_native_platform_because_it_cannot_serve_one() {
        let options = parse_args(&["serve", "counter", "--platform", "native"]).unwrap();
        assert!(options.is_web());
    }

    #[test]
    fn build_targets_the_host_by_default() {
        let options = parse_args(&["build", "peertube"]).unwrap();
        assert_eq!(options.platform, Platform::Native);
        // No --target at all is how cargo is told to use the host.
        assert_eq!(options.target, None);
        assert!(!options.cargo_args().iter().any(|a| a == "--target"));
    }

    #[test]
    fn build_can_target_the_web() {
        let options = parse_args(&["build", "peertube", "--platform", "web"]).unwrap();
        assert_eq!(options.platform, Platform::Web);
        assert_eq!(options.target.as_deref(), Some("wasm32-unknown-unknown"));
        let args = options.cargo_args();
        assert!(args.contains(&"--target".to_string()));
        assert!(args.contains(&"wasm32-unknown-unknown".to_string()));
    }

    #[test]
    fn run_targets_the_host_and_asks_cargo_to_run() {
        let options = parse_args(&["run", "counter"]).unwrap();
        assert_eq!(options.command, Command::Run);
        assert_eq!(options.platform, Platform::Native);
        assert_eq!(options.cargo_args()[0], "run");
    }

    #[test]
    fn run_refuses_the_web_because_a_wasm_binary_cannot_execute() {
        let error = parse_args(&["run", "counter", "--platform", "web"]).unwrap_err();
        assert!(error.contains("serve"), "{error}");
    }

    #[test]
    fn an_unknown_platform_is_named_in_the_error() {
        let error = parse_args(&["build", "--platform", "wii"]).unwrap_err();
        assert!(
            error.contains("--platform expects native or web"),
            "{error}"
        );
    }

    #[test]
    fn an_explicit_target_overrides_the_platform_default() {
        let options = parse_args(&[
            "build",
            "--platform",
            "native",
            "--target",
            "armv7-unknown-linux-gnueabihf",
        ])
        .unwrap();
        assert_eq!(
            options.target.as_deref(),
            Some("armv7-unknown-linux-gnueabihf")
        );
        assert!(
            options
                .cargo_args()
                .contains(&"armv7-unknown-linux-gnueabihf".to_string())
        );
    }

    #[test]
    fn legacy_is_refused_off_the_web() {
        let error = parse_args(&["build", "app", "--legacy"]).unwrap_err();
        assert!(error.contains("--platform web"), "{error}");
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
    fn an_html_path_is_never_taken_for_the_package() {
        let options = parse_args(&["serve", "-e", "webgl", "app/index.html"]).unwrap();
        assert_eq!(options.example.as_deref(), Some("webgl"));
        assert_eq!(options.package, None);
        assert_eq!(options.html, PathBuf::from("app/index.html"));
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
        assert_eq!(options.target.as_deref(), Some("wasm32-wasi"));
    }

    #[test]
    fn an_example_resolves_to_the_package_that_has_it() {
        // The workspace is read for this, only assert it does not blow up.
        let options = parse_args(&["serve", "-e", "webgl"]).unwrap();
        assert_eq!(options.package, None);
        let _ = options.resolve_package();
    }

    #[test]
    fn a_missing_example_names_the_ones_that_exist() {
        let error = missing_example("definitely-not-an-example");
        assert!(error.contains("definitely-not-an-example"));
        if !crate::cargo_metadata::examples().is_empty() {
            assert!(error.contains("it has:"), "{error}");
        }
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
        // No -Zbuild-std, no RUSTFLAGS unless asked: the project config carries them.
        assert!(!args.iter().any(|arg| arg.contains("build-std")));
        assert!(options.cargo_env().is_empty());
    }

    #[test]
    fn legacy_is_off_by_default() {
        let options = parse_args(&["serve"]).unwrap();
        assert!(!options.legacy);
        assert!(!options.cargo_args().iter().any(|a| a.contains("build-std")));
        assert!(options.cargo_env().is_empty());
    }

    #[test]
    fn legacy_adds_the_build_std_and_the_mvp_rustflags() {
        // Neither half is enough on its own: MVP codegen with a precompiled std
        // still leaks post-MVP instructions into the module.
        let options = parse_args(&["build", "app", "--platform", "web", "--legacy"]).unwrap();
        assert!(options.legacy);
        assert!(
            options.cargo_args().contains(&BUILD_STD.to_string()),
            "{:?}",
            options.cargo_args()
        );
        let env = options.cargo_env();
        assert_eq!(env.len(), 1);
        assert_eq!(env[0].0, "RUSTFLAGS");
        assert_eq!(env[0].1, LEGACY_RUSTFLAGS);
    }

    #[test]
    fn legacy_works_on_serve_too() {
        let options = parse_args(&["serve", "-p", "xs-js-sys", "-e", "webgl", "--legacy"]).unwrap();
        assert!(options.legacy);
        assert!(options.cargo_args().contains(&BUILD_STD.to_string()));
        assert_eq!(options.cargo_env()[0].0, "RUSTFLAGS");
    }

    #[test]
    fn legacy_leaves_the_rest_of_the_build_alone() {
        let plain = parse_args(&["build", "app", "--platform", "web", "--release"]).unwrap();
        let legacy =
            parse_args(&["build", "app", "--platform", "web", "--release", "--legacy"]).unwrap();
        // One argument longer, and it is the build-std one.
        assert_eq!(legacy.cargo_args().len(), plain.cargo_args().len() + 1);
        assert!(plain.cargo_args().contains(&"--release".to_string()));
        assert!(legacy.cargo_args().contains(&"--release".to_string()));
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
