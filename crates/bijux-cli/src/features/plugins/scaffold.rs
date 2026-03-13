//! Plugin scaffold generation helpers.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::Result;

use crate::api::version::runtime_semver;

use super::{is_reserved_namespace, parse_manifest_v1, validate_manifest, RESERVED_NAMESPACES};

fn is_safe_scaffold_path(path: &Path) -> bool {
    !path.components().any(|component| matches!(component, Component::ParentDir))
}

fn scaffold_manifest_json(plugin_kind: &str, namespace: &str) -> String {
    format!(
        "{{\n  \"name\": \"{}\",\n  \"version\": \"0.1.0\",\n  \"schema_version\": \"v1\",\n  \"manifest_version\": \"v1\",\n  \"compatibility\": {{ \"min_inclusive\": \"{}\", \"max_exclusive\": null }},\n  \"namespace\": \"{}\",\n  \"kind\": \"{}\",\n  \"aliases\": [],\n  \"entrypoint\": \"{}\",\n  \"capabilities\": []\n}}\n",
        namespace,
        runtime_semver(),
        namespace,
        plugin_kind,
        "plugin:main",
    )
}

fn scaffold_manifest_kind(kind: &str) -> Result<&'static str> {
    match kind {
        "python" => Ok("python"),
        "rust" => Ok("delegated"),
        _ => anyhow::bail!("plugin scaffold kind must be one of: python, rust"),
    }
}

pub(crate) fn scaffold_plugin_layout(
    base_dir: &Path,
    kind: &str,
    namespace: &str,
    force: bool,
) -> Result<PathBuf> {
    if is_reserved_namespace(namespace, &[]) {
        anyhow::bail!("plugin namespace is reserved: {namespace}");
    }
    if !namespace.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        anyhow::bail!("plugin namespace must be lowercase kebab-case");
    }
    if !is_safe_scaffold_path(base_dir) {
        anyhow::bail!("scaffold path is unsafe");
    }
    let plugin_kind = scaffold_manifest_kind(kind)?;
    if base_dir.exists() {
        if !force {
            anyhow::bail!("scaffold path already exists; pass --force to overwrite");
        }

        if base_dir.is_dir() {
            fs::remove_dir_all(base_dir)?;
        } else {
            fs::remove_file(base_dir)?;
        }
    }

    fs::create_dir_all(base_dir)?;
    let manifest_path = base_dir.join("plugin.manifest.json");
    fs::write(&manifest_path, scaffold_manifest_json(plugin_kind, namespace))?;
    if kind == "python" {
        fs::write(
            base_dir.join("plugin.py"),
            "def main(argv: list[str]) -> dict:\n    return {\"status\": \"ok\", \"argv\": argv}\n",
        )?;
    } else {
        fs::write(
            base_dir.join("plugin.py"),
            "def main(argv: list[str]) -> dict:\n    return {\"status\": \"ok\", \"argv\": argv, \"bridge\": \"replace plugin.py with your Rust bridge entrypoint\"}\n",
        )?;
        fs::create_dir_all(base_dir.join("src"))?;
        fs::write(
            base_dir.join("src/lib.rs"),
            "pub fn main(argv: &[String]) -> String { format!(\"ok {}\", argv.len()) }\n",
        )?;
    }

    // Shared validation step: generated manifest must pass plugin parser.
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let manifest = parse_manifest_v1(&manifest_text)?;
    let _ = validate_manifest(manifest, runtime_semver(), RESERVED_NAMESPACES)?;

    Ok(manifest_path)
}
