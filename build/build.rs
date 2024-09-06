use std::env::var;
use std::path::Path;

use mavspec::rust::gen::BuildHelper;

fn main() {
    // Assume that your library and `message_definitions` are both in the root of your project.
    let sources = vec!["./serpe-dialect"];
    // Output path
    let destination = Path::new(&var("OUT_DIR").unwrap()).join("mavlink");
    // Path to your `Cargo.toml` manifest
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

    // Parse XML definitions and generate Rust code
    BuildHelper::builder(&destination)
        .set_sources(&sources)
        .set_manifest_path(&manifest_path)
        .set_serde(false)
        .generate()
        .unwrap();
}
