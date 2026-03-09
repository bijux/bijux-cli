#![forbid(unsafe_code)]
//! Generate PluginManifestV1 JSON Schema artifact.

use std::env;
use std::fs;
use std::path::PathBuf;

use bijux_cli_contracts::schema::plugin_manifest_v1_schema;
use schemars as _;
use proptest as _;
use semver as _;
use serde as _;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("docs/constitution/schemas/plugin-manifest-v1.schema.json")
    });

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }

    let schema = plugin_manifest_v1_schema();
    let rendered = serde_json::to_string_pretty(&schema)?;
    fs::write(&out, rendered)?;
    println!("wrote {}", out.display());
    Ok(())
}
