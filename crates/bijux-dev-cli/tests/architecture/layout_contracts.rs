#![forbid(unsafe_code)]
//! Filesystem layout contracts for bijux-dev-cli architecture.

use std::fs;
use std::path::{Path, PathBuf};

fn collect_files_recursive(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}

#[test]
fn src_tree_matches_cli_contracts_suites_reports_infra_schema_layout() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for directory in [
        "src/cli",
        "src/contracts",
        "src/contracts/status",
        "src/contracts/maintenance",
        "src/suites",
        "src/reports",
        "src/infra",
        "src/schema",
    ] {
        assert!(
            crate_root.join(directory).is_dir(),
            "missing directory {directory}"
        );
    }

    for file in [
        "src/cli/args.rs",
        "src/cli/dispatch.rs",
        "src/cli/workspace.rs",
        "src/cli/routes/root.rs",
        "src/cli/routes/maintenance.rs",
        "src/cli/routes/release.rs",
        "src/cli/routes/evidence.rs",
        "src/cli/routes/config.rs",
        "src/cli/routes/python.rs",
        "src/cli/routes/rustdoc.rs",
        "src/contracts/status/model.rs",
        "src/contracts/status/inventory.rs",
        "src/contracts/status/run.rs",
        "src/contracts/maintenance/inventory.rs",
        "src/contracts/maintenance/compliance.rs",
        "src/contracts/maintenance/generators.rs",
        "src/suites/control_plane/catalog.rs",
        "src/suites/control_plane/run.rs",
        "src/suites/control_plane/orchestration.rs",
        "src/suites/control_plane/ownership.rs",
        "src/suites/control_plane/stale_artifacts.rs",
        "src/suites/runtime/catalog.rs",
        "src/suites/runtime/run.rs",
        "src/suites/runtime/config_surface.rs",
        "src/suites/runtime/cross_surface.rs",
        "src/suites/runtime/install_runtime.rs",
        "src/suites/runtime/repl_bridge.rs",
        "src/suites/quality/catalog.rs",
        "src/suites/quality/run.rs",
        "src/suites/quality/command_surface.rs",
        "src/suites/quality/release_evidence.rs",
        "src/suites/quality/state_laws.rs",
        "src/suites/quality/plugin_quality.rs",
        "src/suites/resilience/catalog.rs",
        "src/suites/resilience/run.rs",
        "src/suites/resilience/corruption_campaigns.rs",
        "src/suites/resilience/fs_process_adversarial.rs",
        "src/suites/resilience/parser_fuzz.rs",
        "src/reports/cockpit.rs",
        "src/reports/release.rs",
        "src/reports/evidence.rs",
        "src/reports/config.rs",
        "src/reports/python.rs",
        "src/reports/runtime_surface.rs",
        "src/reports/repository_health.rs",
        "src/infra/artifacts.rs",
        "src/infra/fs.rs",
        "src/infra/process.rs",
        "src/infra/clock.rs",
        "src/schema/command_registry.rs",
        "src/schema/report_envelope.rs",
    ] {
        assert!(crate_root.join(file).is_file(), "missing file {file}");
    }
}

#[test]
fn legacy_namespaces_and_legacy_suffix_files_are_removed() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for legacy in [
        "src/app",
        "src/dispatch.rs",
        "src/platform",
        "src/infrastructure",
        "src/status_contracts",
        "src/contracts/native",
        "src/contracts/maintenance/shared.rs",
    ] {
        assert!(
            !crate_root.join(legacy).exists(),
            "legacy path must not exist: {legacy}"
        );
    }

    let suites_root = crate_root.join("src/suites");
    let mut files = Vec::<PathBuf>::new();
    collect_files_recursive(&suites_root, &mut files);
    for file in files {
        if file.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        assert!(
            !name.ends_with("_executor.rs") && !name.ends_with("_spec.rs"),
            "legacy suffix file must be removed: {}",
            file.display()
        );
    }
}
