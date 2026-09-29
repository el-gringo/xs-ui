use std::path::{Path, PathBuf};

use crate::{html::Html, options::Options};

/// Name the wasm is served under, the page and the hot reload only ever see
/// this.
pub const WASM_NAME: &str = "app.wasm";

/// Builds the project and everything the page needs into `--dist`, ready to be
/// served as static files.
///
/// The page is copied next to it with the wasm loader injected and without the
/// dev server client: there is no websocket to talk to once it is deployed.
pub fn run(options: &Options) {
    let package = crate::resolve_package(options);
    let metadata = crate::cargo_metadata::get_config();

    let status = std::process::Command::new("cargo")
        .args(options.cargo_args())
        .status()
        .unwrap_or_else(|error| {
            eprintln!("error: failed to run cargo: {error}");
            std::process::exit(1);
        });
    if !status.success() {
        eprintln!("error: cargo build failed");
        std::process::exit(1);
    }

    let html = match Html::read(&options.html) {
        Ok(html) => html,
        Err(error) => fail(error),
    };
    let wasm = options.wasm_path(&metadata.target_directory);
    let dist = options.dist.clone();

    if let Err(error) = std::fs::create_dir_all(&dist) {
        fail(format!("unable to create {}: {error}", dist.display()));
    }
    // The assets the page refers to, resolved next to the html file.
    if let Err(error) = copy_dir(html.directory(), &dist) {
        fail(format!(
            "unable to copy the assets of {}: {error}",
            html.directory().display()
        ));
    }
    let page = html.render(WASM_NAME, false);
    if let Err(error) = std::fs::write(dist.join("index.html"), page) {
        fail(format!("unable to write the index: {error}"));
    }
    if let Err(error) = std::fs::copy(&wasm, dist.join(WASM_NAME)) {
        fail(format!("unable to copy {}: {error}", wasm.display()));
    }
    println!(
        "built {package} (html: {}) into {}",
        options.html.display(),
        dist.display()
    );
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<PathBuf> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(to.to_path_buf())
}

fn fail(error: String) -> ! {
    eprintln!("error: {error}");
    std::process::exit(1);
}
