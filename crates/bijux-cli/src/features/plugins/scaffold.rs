//! Plugin scaffold generation helpers.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use semver::{Prerelease, Version};

use super::{
    is_reserved_namespace, parse_manifest_v2, validate_manifest, validate_namespace_text,
    RESERVED_NAMESPACES,
};

const SCAFFOLD_PLUGIN_VERSION: &str = "0.1.0";
const RUST_SCAFFOLD_ENTRYPOINT: &str = "plugin-entrypoint";

fn is_safe_scaffold_path(path: &Path) -> bool {
    !path.components().any(|component| matches!(component, Component::ParentDir))
}

fn scaffold_compatibility_window() -> Result<(String, String)> {
    let release_line = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| anyhow::anyhow!("runtime semver is invalid: {error}"))?;

    let mut min = Version::new(release_line.major, release_line.minor, release_line.patch);
    if !release_line.pre.is_empty() {
        let channel = release_line
            .pre
            .as_str()
            .split('.')
            .next()
            .expect("non-empty prerelease has a first identifier");
        min.pre = Prerelease::new(channel)
            .map_err(|error| anyhow::anyhow!("runtime prerelease channel is invalid: {error}"))?;
    }

    let max = if release_line.major == 0 {
        Version::new(0, release_line.minor.saturating_add(1), 0)
    } else {
        Version::new(release_line.major.saturating_add(1), 0, 0)
    };
    Ok((min.to_string(), max.to_string()))
}

fn scaffold_manifest_json(plugin_kind: &str, entrypoint: &str, namespace: &str) -> Result<String> {
    let (min_inclusive, max_exclusive) = scaffold_compatibility_window()?;
    Ok(format!(
        "{{\n  \"name\": \"{}\",\n  \"version\": \"{}\",\n  \"schema_version\": \"v2\",\n  \"manifest_version\": \"v2\",\n  \"compatibility\": {{ \"min_inclusive\": \"{}\", \"max_exclusive\": \"{}\" }},\n  \"namespace\": \"{}\",\n  \"kind\": \"{}\",\n  \"trust_class\": \"community\",\n  \"aliases\": [],\n  \"entrypoint\": \"{}\",\n  \"capabilities\": []\n}}\n",
        namespace,
        SCAFFOLD_PLUGIN_VERSION,
        min_inclusive,
        max_exclusive,
        namespace,
        plugin_kind,
        entrypoint,
    ))
}

fn scaffold_manifest_contract(kind: &str) -> Result<(&'static str, &'static str)> {
    match kind {
        "python" => Ok(("python", "plugin:main")),
        "rust" => Ok(("external-exec", RUST_SCAFFOLD_ENTRYPOINT)),
        _ => anyhow::bail!("plugin scaffold kind must be one of: python, rust"),
    }
}

fn rust_entrypoint_script(binary_name: &str, namespace: &str) -> String {
    format!(
        "#!/usr/bin/env sh\nset -eu\n\nSCRIPT_DIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\ncd \"$SCRIPT_DIR\"\n\nexport CARGO_TARGET_DIR=\"$SCRIPT_DIR/target\"\nBIN_PATH=\"$CARGO_TARGET_DIR/debug/{binary_name}\"\nLOCK_PATH=\"$SCRIPT_DIR/Cargo.lock\"\nneeds_build=0\nneeds_lock_refresh=0\nif [ ! -f \"$LOCK_PATH\" ]; then\n  needs_lock_refresh=1\nelif [ \"$SCRIPT_DIR/Cargo.toml\" -nt \"$LOCK_PATH\" ]; then\n  needs_lock_refresh=1\nfi\nif [ ! -x \"$BIN_PATH\" ]; then\n  needs_build=1\nelif [ \"$SCRIPT_DIR/Cargo.toml\" -nt \"$BIN_PATH\" ]; then\n  needs_build=1\nelif [ \"$LOCK_PATH\" -nt \"$BIN_PATH\" ]; then\n  needs_build=1\nelif [ -d \"$SCRIPT_DIR/src\" ] && find \"$SCRIPT_DIR/src\" -type f -name '*.rs' -newer \"$BIN_PATH\" -print -quit | grep -q .; then\n  needs_build=1\nfi\n\nif [ \"$needs_build\" -eq 1 ] || [ \"$needs_lock_refresh\" -eq 1 ]; then\n  if ! command -v cargo >/dev/null 2>&1; then\n    echo \"cargo is required to build the {namespace} plugin binary\" >&2\n    exit 1\n  fi\nfi\nif [ \"$needs_lock_refresh\" -eq 1 ]; then\n  lock_stderr=$(mktemp)\n  if ! cargo generate-lockfile 1>/dev/null 2>\"$lock_stderr\"; then\n    cat \"$lock_stderr\" >&2\n    rm -f \"$lock_stderr\"\n    exit 1\n  fi\n  rm -f \"$lock_stderr\"\nfi\nif [ \"$needs_build\" -eq 1 ]; then\n  build_stderr=$(mktemp)\n  if ! cargo build --quiet --locked 2>\"$build_stderr\"; then\n    cat \"$build_stderr\" >&2\n    rm -f \"$build_stderr\"\n    exit 1\n  fi\n  rm -f \"$build_stderr\"\nfi\n\nexec \"$BIN_PATH\" \"$@\"\n"
    )
}

#[cfg(unix)]
fn mark_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) -> Result<()> {
    Ok(())
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
    validate_namespace_text(namespace).map_err(anyhow::Error::from)?;
    if !is_safe_scaffold_path(base_dir) {
        anyhow::bail!("scaffold path is unsafe");
    }
    let (plugin_kind, entrypoint) = scaffold_manifest_contract(kind)?;
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
    fs::write(&manifest_path, scaffold_manifest_json(plugin_kind, entrypoint, namespace)?)?;
    if kind == "python" {
        fs::write(
            base_dir.join("plugin.py"),
            "def main(argv: list[str]) -> dict:\n    return {\"status\": \"ok\", \"argv\": argv}\n",
        )?;
    } else {
        let cargo_package_name = namespace;
        let cargo_module_name = namespace.replace('-', "_");
        fs::write(
            base_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{cargo_package_name}\"\nversion = \"{SCAFFOLD_PLUGIN_VERSION}\"\nedition = \"2021\"\nlicense = \"Apache-2.0\"\ndescription = \"Rust executable plugin for {namespace}\"\n\n[workspace]\n\n[lib]\nname = \"{cargo_module_name}\"\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"{cargo_package_name}\"\npath = \"src/main.rs\"\n\n[dependencies]\nserde_json = \"1\"\n"
            ),
        )?;
        let entrypoint_path = base_dir.join(RUST_SCAFFOLD_ENTRYPOINT);
        fs::write(&entrypoint_path, rust_entrypoint_script(cargo_package_name, namespace))?;
        mark_executable(&entrypoint_path)?;
        fs::create_dir_all(base_dir.join("src"))?;
        fs::write(
            base_dir.join("src/lib.rs"),
            format!(
                "use serde_json::{{json, Value}};\n\npub fn run(argv: &[String]) -> Value {{\n    json!({{\"status\": \"ok\", \"namespace\": \"{namespace}\", \"argv\": argv}})\n}}\n\npub fn help_text() -> &'static str {{\n    \"Usage: {namespace} [ARGS]\\n\\nRuns the {namespace} Rust plugin entrypoint.\"\n}}\n"
            ),
        )?;
        fs::write(
            base_dir.join("src/main.rs"),
            format!(
                "use std::process::ExitCode;\n\nfn main() -> ExitCode {{\n    let argv = std::env::args().skip(1).collect::<Vec<_>>();\n    if argv.iter().any(|arg| matches!(arg.as_str(), \"--help\" | \"-h\")) {{\n        println!(\"{{}}\", {rust_module}::help_text());\n        return ExitCode::SUCCESS;\n    }}\n\n    match serde_json::to_string_pretty(&{rust_module}::run(&argv)) {{\n        Ok(rendered) => {{\n            println!(\"{{rendered}}\");\n            ExitCode::SUCCESS\n        }}\n        Err(error) => {{\n            eprintln!(\"failed to render plugin payload: {{error}}\");\n            ExitCode::from(1)\n        }}\n    }}\n}}\n",
                rust_module = cargo_module_name
            ),
        )?;
    }

    // Shared validation step: generated manifest must pass plugin parser.
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let manifest = parse_manifest_v2(&manifest_text)?;
    let _ = validate_manifest(manifest, env!("CARGO_PKG_VERSION"), RESERVED_NAMESPACES)?;

    Ok(manifest_path)
}
