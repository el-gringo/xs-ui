use nanoserde::DeJson;

// #[derive(Debug, DeJson)]
// pub struct CargoMetadataPackageTarget {
//     pub src_path: String
// }

// #[derive(Debug, DeJson)]
// pub struct CargoMetadataPackage {
//     pub name: String,
//     // pub targets: Vec<CargoMetadataPackageTarget>
// }

#[derive(Debug, DeJson)]
pub struct CargoMetadata {
    // pub packages: Vec<CargoMetadataPackage>,
    pub target_directory: String,
}

pub fn get_config() -> CargoMetadata {
    metadata()
}

/// Binaries of the packages cargo builds by default, in the order it builds
/// them. The first one is what the CLI serves when no package is given.
pub fn default_bins() -> Vec<String> {
    // A package with several bins is ambiguous here, its binaries share the
    // package name so only the first is reported.
    default_packages()
        .into_iter()
        .filter(|package| package.targets.iter().any(Target::is_bin))
        .map(|package| package.name.clone())
        .take(1)
        .collect()
}

fn metadata_json() -> Option<Metadata> {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .ok()?;
    <Metadata as DeJson>::deserialize_json(&String::from_utf8_lossy(&output.stdout)).ok()
}

fn metadata() -> CargoMetadata {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("failed to run cargo metadata");
    <CargoMetadata as DeJson>::deserialize_json(&String::from_utf8_lossy(&output.stdout))
        .expect("failed to parse the cargo metadata")
}

#[derive(Debug, DeJson)]
struct Metadata {
    packages: Vec<Package>,
    /// The members cargo builds by default, the ones `xs` picks from when it
    /// has to choose a binary.
    #[nserde(default)]
    workspace_default_members: Vec<String>,
    /// Every member, including the ones the default list leaves out.
    #[nserde(default)]
    workspace_members: Vec<String>,
}

#[derive(Debug, DeJson, Clone)]
struct Package {
    name: String,
    id: String,
    targets: Vec<Target>,
}

/// The target name lives next to its kind in the metadata, nanoserde fills it
/// from the `name` field of the same object.
fn target_name(target: &Target) -> String {
    target.name.clone()
}

#[derive(Debug, DeJson, Clone)]
struct Target {
    name: String,
    kind: Vec<String>,
    #[nserde(default)]
    crate_types: Vec<String>,
}

impl Target {
    fn is_bin(&self) -> bool {
        self.kind.iter().any(|kind| kind == "bin")
    }

    fn is_example(&self) -> bool {
        self.kind.iter().any(|kind| kind == "example")
    }

    /// An example cargo can turn into a wasm: the ones that are declared as
    /// libraries are helpers, `cargo build --example` has no entry point.
    fn is_wasm_example(&self) -> bool {
        self.is_example() && self.crate_types.iter().any(|kind| kind == "bin")
    }
}

/// Packages of the workspace that declare an example called `name`.
pub fn packages_with_example(name: &str) -> Vec<String> {
    all_packages()
        .into_iter()
        .filter(|package| {
            package
                .targets
                .iter()
                .any(|target| target.is_wasm_example() && target_name(target) == name)
        })
        .map(|package| package.name.clone())
        .collect()
}

/// Every buildable example of the workspace, for the error message.
pub fn examples() -> Vec<(String, Vec<String>)> {
    all_packages()
        .into_iter()
        .filter_map(|package| {
            let mut names: Vec<String> = package
                .targets
                .iter()
                .filter(|target| target.is_wasm_example())
                .map(target_name)
                .collect();
            if names.is_empty() {
                return None;
            }
            names.sort();
            Some((package.name.clone(), names))
        })
        .collect()
}

/// The packages the default member list names, the ones a bare `xs` serves.
fn default_packages() -> Vec<Package> {
    let Some(metadata) = metadata_json() else {
        return Vec::new();
    };
    members(&metadata, &metadata.workspace_default_members)
}

/// Every member of the workspace: an example can live outside the default list.
fn all_packages() -> Vec<Package> {
    let Some(metadata) = metadata_json() else {
        return Vec::new();
    };
    members(&metadata, &metadata.workspace_members)
}

fn members(metadata: &Metadata, ids: &[String]) -> Vec<Package> {
    let ids: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();
    metadata
        .packages
        .iter()
        .filter(|package| ids.contains(&package.id.as_str()))
        .cloned()
        .collect()
}
