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
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    let json = String::from_utf8_lossy(&output.stdout);
    let Ok(metadata) = <Metadata as DeJson>::deserialize_json(&json) else {
        return Vec::new();
    };
    let defaults: Vec<&str> = metadata
        .workspace_default_members
        .iter()
        .map(|id| id.as_str())
        .collect();
    metadata
        .packages
        .iter()
        .filter(|package| defaults.contains(&package.id.as_str()))
        // A package with several bins is ambiguous here, its binaries share the
        // package name so only the first is reported.
        .flat_map(|package| {
            package
                .targets
                .iter()
                .filter(|target| target.is_bin())
                .map(|_| package.name.clone())
                .take(1)
        })
        .collect()
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
    #[nserde(default)]
    workspace_default_members: Vec<String>,
}

#[derive(Debug, DeJson)]
struct Package {
    name: String,
    id: String,
    targets: Vec<Target>,
}

#[derive(Debug, DeJson)]
struct Target {
    kind: Vec<String>,
}

impl Target {
    fn is_bin(&self) -> bool {
        self.kind.iter().any(|kind| kind == "bin")
    }
}
