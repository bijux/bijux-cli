//! Maintainer package health report assembly.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

fn python_entrypoints(workspace_root: &Path) -> Vec<String> {
    let pyproject = workspace_root.join("crates/bijux-cli-python/pyproject.toml");
    let Some(text) = fs::read_to_string(pyproject).ok() else {
        return Vec::new();
    };
    let Ok(payload) = toml::from_str::<toml::Table>(&text) else {
        return Vec::new();
    };
    payload
        .get("project")
        .and_then(toml::Value::as_table)
        .and_then(|project| project.get("scripts"))
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}

fn package_entrypoints(runtime_identity_report: &Value, workspace_root: &Path) -> Vec<Value> {
    let canonical_binary = runtime_identity_report
        .get("canonical_user_binary")
        .and_then(Value::as_str)
        .unwrap_or("bijux");
    let cargo_package = runtime_identity_report
        .get("package_channels")
        .and_then(|value| value.get("cargo"))
        .and_then(|value| value.get("canonical"))
        .and_then(Value::as_str)
        .unwrap_or("bijux-cli");
    let pip_package = runtime_identity_report
        .get("package_channels")
        .and_then(|value| value.get("pip"))
        .and_then(|value| value.get("canonical"))
        .and_then(Value::as_str)
        .unwrap_or("bijux-cli");
    let rust_target = runtime_identity_report
        .get("entrypoints")
        .and_then(|value| value.get("binary"))
        .and_then(Value::as_str)
        .unwrap_or("crates/bijux-cli/src/bin/bijux.rs");

    let mut rows = vec![json!({
        "channel": "cargo",
        "package": cargo_package,
        "entrypoint": canonical_binary,
        "target": rust_target,
    })];

    let pyproject = workspace_root.join("crates/bijux-cli-python/pyproject.toml");
    let python_scripts = python_entrypoints(workspace_root);
    if let Ok(text) = fs::read_to_string(pyproject) {
        if let Ok(payload) = toml::from_str::<toml::Table>(&text) {
            if let Some(scripts) = payload
                .get("project")
                .and_then(toml::Value::as_table)
                .and_then(|project| project.get("scripts"))
                .and_then(toml::Value::as_table)
            {
                for entrypoint in python_scripts {
                    if let Some(target) = scripts.get(&entrypoint).and_then(toml::Value::as_str) {
                        rows.push(json!({
                            "channel": "pip",
                            "package": pip_package,
                            "entrypoint": entrypoint,
                            "target": target,
                        }));
                    }
                }
            }
        }
    }

    rows
}

fn runtime_identity_rules(runtime_identity_report: &Value, workspace_root: &Path) -> Value {
    let python_entrypoints = python_entrypoints(workspace_root);
    json!({
        "canonical_user_binary": runtime_identity_report
            .get("canonical_user_binary")
            .cloned()
            .unwrap_or_else(|| json!("bijux")),
        "public_runtime_binary_names": runtime_identity_report
            .get("public_runtime_binary_names")
            .cloned()
            .unwrap_or_else(|| json!(["bijux"])),
        "package_channels": runtime_identity_report
            .get("package_channels")
            .cloned()
            .unwrap_or_else(|| json!({})),
        "python_package_entrypoints": python_entrypoints,
        "python_package_points_users_to_bijux": runtime_identity_report
            .get("canonical_user_binary")
            .and_then(Value::as_str)
            .is_some_and(|binary| python_entrypoints.iter().any(|entrypoint| entrypoint == binary)),
    })
}

/// Builds the maintainer package health report payload.
#[must_use]
pub fn build_report(workspace_root: &Path, runtime_identity_report: Value) -> Value {
    let assumptions = vec![
        "config/history/plugins state defaults under HOME/.bijux unless explicit overrides are set",
        "XDG-style HOME locations are treated as regular HOME roots for compatibility paths",
        "PATH order decides active bijux binary and all ambiguity diagnostics derive from that order",
        "completion files are generated under shell-specific directories derived from HOME",
        "state bootstrap must create missing directories and report explicit errors for unwritable roots",
    ];
    json!({
        "evidence_ids": ["EVIDENCE-1003-INSTALL-NEUTRALITY"],
        "package_entrypoints": package_entrypoints(&runtime_identity_report, workspace_root),
        "runtime_identity_rules": runtime_identity_rules(&runtime_identity_report, workspace_root),
        "install_state_assumptions": assumptions,
        "install_state_assumption_help": "Use `bijux dev cli package-health --format json` to audit install-state assumptions and entrypoint contracts.",
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;

    use super::build_report;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "bijux-package-health-{name}-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        fs::create_dir_all(root.join("crates/bijux-cli-python")).expect("mkdir");
        root
    }

    #[test]
    fn package_health_report_shape_is_stable() {
        let root = temp_root("shape");
        fs::write(
            root.join("crates/bijux-cli-python/pyproject.toml"),
            "[project.scripts]\nbijux = \"bijux_cli_py.cli:main\"\n",
        )
        .expect("write pyproject");
        let report = build_report(
            &root,
            json!({
                "canonical_user_binary": "bijux",
                "public_runtime_binary_names": ["bijux"],
                "package_channels": {"cargo":{"canonical":"bijux-cli"},"pip":{"canonical":"bijux-cli"}},
                "entrypoints": {"binary": "crates/bijux-cli/src/bin/bijux.rs"},
            }),
        );
        assert!(report.get("package_entrypoints").is_some());
        assert!(report.get("runtime_identity_rules").is_some());
        assert!(report.get("install_state_assumptions").is_some());
    }
}
