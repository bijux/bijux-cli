//! Maintainer runtime identity report assembly.

use serde_json::{json, Value};

/// Canonical executable name.
pub const CANONICAL_EXECUTABLE: &str = "bijux";

/// Install diagnostics report used by runtime identity assembly.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct InstallHealthReport {
    /// Active binary used for invocation.
    pub active_binary: Option<String>,
    /// All discovered binaries named `bijux` in PATH order.
    pub path_binaries: Vec<String>,
    /// Whether multiple binaries are discovered in PATH order.
    pub has_path_shadowing: bool,
    /// Whether installs appear to exist across multiple ecosystems.
    pub has_duplicate_installs: bool,
    /// Wrapper maintenance that no longer point to an existing runtime.
    pub stale_wrapper_maintenance: Vec<String>,
    /// Whether wheel and runtime binary versions differ.
    pub has_mismatched_wheel_binary_versions: bool,
    /// Legacy installer wrappers that could shadow canonical runtime.
    pub legacy_installer_conflicts: Vec<String>,
    /// Whether configured active binary path is missing from disk.
    pub active_binary_missing: bool,
    /// Whether configured active binary path is a broken symlink.
    pub broken_symlink_active_binary: bool,
}

fn detect_install_source(active_binary: Option<&str>) -> &'static str {
    let Some(path) = active_binary else {
        return "unknown";
    };
    if path.contains(".cargo") {
        "cargo"
    } else if path.contains("pipx") {
        "pipx"
    } else if path.contains("site-packages") || path.contains("venv") || path.contains(".venv") {
        "pip"
    } else if path.contains("homebrew") || path.contains("/brew/") {
        "homebrew"
    } else {
        "unknown"
    }
}

fn is_canonical_active_path(active_binary: Option<&str>) -> bool {
    active_binary
        .map(|path| {
            path.ends_with(&format!("/{CANONICAL_EXECUTABLE}")) || path == CANONICAL_EXECUTABLE
        })
        .unwrap_or(false)
}

/// Inputs required to assemble runtime identity report payload.
#[derive(Debug, Clone)]
pub struct RuntimeIdentityInput {
    /// Install diagnostics from low-level install services.
    pub install_report: InstallHealthReport,
    /// Whether python bridge support is available.
    pub python_bridge_supported: bool,
    /// Cargo canonical package name.
    pub cargo_canonical_package: String,
    /// Cargo compatibility package name.
    pub cargo_compat_package: String,
    /// Pip canonical package name.
    pub pip_canonical_package: String,
    /// Pip compatibility package name.
    pub pip_compat_package: String,
    /// Canonical crate package name.
    pub canonical_crate_name: String,
}

/// Builds the maintainer runtime identity report payload.
#[must_use]
pub fn build_report(input: RuntimeIdentityInput) -> Value {
    let install_source = detect_install_source(input.install_report.active_binary.as_deref());
    let is_shadowed = input.install_report.has_path_shadowing;
    let is_ambiguous_active_binary = input.install_report.path_binaries.len() > 1;
    let is_canonical_path = is_canonical_active_path(input.install_report.active_binary.as_deref());

    json!({
        "runtime_truth_default": "bijux dev cli runtime-identity",
        "evidence_ids": ["EVIDENCE-1004-RUNTIME-IDENTITY"],
        "runtime": "rust-foundation",
        "schema": "runtime-identity-v1",
        "public_runtime_binary_names": [CANONICAL_EXECUTABLE],
        "secondary_public_runtime_binary_names": [],
        "canonical_user_binary": CANONICAL_EXECUTABLE,
        "active_binary": input.install_report.active_binary,
        "install_source": install_source,
        "active_path_is_canonical_name": is_canonical_path,
        "active_path_is_shadowed": is_shadowed,
        "active_binary_selection_is_ambiguous": is_ambiguous_active_binary,
        "path_binaries": input.install_report.path_binaries,
        "diagnostics": {
            "duplicate_install_detected": input.install_report.has_duplicate_installs,
            "mixed_pip_cargo_install_detected": input.install_report.has_duplicate_installs,
            "path_shadowing_detected": input.install_report.has_path_shadowing,
            "stale_wrapper_detected": !input.install_report.stale_wrapper_maintenance.is_empty(),
            "stale_wrapper_maintenance": input.install_report.stale_wrapper_maintenance,
            "mismatched_wheel_binary_versions": input.install_report.has_mismatched_wheel_binary_versions,
            "active_binary_mismatch_detected": input.install_report.has_mismatched_wheel_binary_versions,
            "active_binary_missing": input.install_report.active_binary_missing,
            "broken_symlink_active_binary": input.install_report.broken_symlink_active_binary,
            "python_bridge_supported": input.python_bridge_supported,
            "legacy_installer_conflicts": input.install_report.legacy_installer_conflicts,
        },
        "entrypoints": {
            "binary": "crates/bijux-cli/src/bin/bijux.rs",
            "core": "bijux_cli::api::runtime::run_app",
            "python_bridge": "bijux_cli_python::bindings::execution_facade_api",
        },
        "package_channels": {
            "cargo": {
                "canonical": input.cargo_canonical_package,
                "compatibility": input.cargo_compat_package,
            },
            "pip": {
                "canonical": input.pip_canonical_package,
                "compatibility": input.pip_compat_package,
            },
            "canonical_crate_name": input.canonical_crate_name,
        },
        "text_summary": [
            format!("canonical user binary: {CANONICAL_EXECUTABLE}"),
            format!("active executable: {}", input.install_report.active_binary.clone().unwrap_or_else(|| "not-found".to_string())),
            format!("install source: {install_source}"),
            format!("path shadowing: {}", if input.install_report.has_path_shadowing { "detected" } else { "not-detected" }),
            format!("duplicate installs: {}", if input.install_report.has_duplicate_installs { "detected" } else { "not-detected" }),
            format!("stale wrappers: {}", if input.install_report.stale_wrapper_maintenance.is_empty() { "not-detected" } else { "detected" }),
            format!("python bridge support: {}", if input.python_bridge_supported { "available" } else { "missing" }),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::InstallHealthReport;
    use super::{build_report, RuntimeIdentityInput};

    #[test]
    fn runtime_identity_report_shape_is_stable() {
        let input = RuntimeIdentityInput {
            install_report: InstallHealthReport {
                active_binary: Some("/tmp/bijux".to_string()),
                path_binaries: vec!["/tmp/bijux".to_string()],
                has_path_shadowing: false,
                has_duplicate_installs: false,
                stale_wrapper_maintenance: Vec::new(),
                has_mismatched_wheel_binary_versions: false,
                legacy_installer_conflicts: Vec::new(),
                active_binary_missing: false,
                broken_symlink_active_binary: false,
            },
            python_bridge_supported: true,
            cargo_canonical_package: "bijux-cli".to_string(),
            cargo_compat_package: "bijux-cli".to_string(),
            pip_canonical_package: "bijux-cli".to_string(),
            pip_compat_package: "bijux-cli".to_string(),
            canonical_crate_name: "bijux-cli".to_string(),
        };
        let report = build_report(input);
        assert!(report.get("diagnostics").is_some());
        assert!(report.get("entrypoints").is_some());
        assert!(report.get("package_channels").is_some());
    }
}
