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
    let args = vec!["metadata", "--format-version", "1"];
    let output = std::process::Command::new("cargo")
        .args(args)
        .output()
        .expect("failed to spawn cargo watch");
    let json = &String::from_utf8_lossy(&output.stdout);
    let config: CargoMetadata = DeJson::deserialize_json(json).unwrap();
    config
}
