#![forbid(unsafe_code)]
//! Filesystem layout contracts for bijux-dev architecture.

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
        "src/maintainer/cli",
        "src/maintainer/contracts",
        "src/maintainer/contracts/status",
        "src/maintainer/contracts/maintenance",
        "src/maintainer/suites",
        "src/maintainer/reports",
        "src/maintainer/infra",
        "src/maintainer/schema",
    ] {
        assert!(crate_root.join(directory).is_dir(), "missing directory {directory}");
    }

    for file in [
        "src/maintainer/cli/args.rs",
        "src/maintainer/cli/dispatch.rs",
        "src/maintainer/cli/workspace.rs",
        "src/maintainer/cli/routes/root.rs",
        "src/maintainer/cli/routes/docs.rs",
        "src/maintainer/cli/routes/maintenance.rs",
        "src/maintainer/cli/routes/release.rs",
        "src/maintainer/cli/routes/evidence.rs",
        "src/maintainer/cli/routes/config.rs",
        "src/maintainer/cli/routes/python.rs",
        "src/maintainer/cli/routes/rustdoc.rs",
        "src/maintainer/contracts/status/model.rs",
        "src/maintainer/contracts/status/inventory.rs",
        "src/maintainer/contracts/status/run.rs",
        "src/maintainer/contracts/maintenance/inventory.rs",
        "src/maintainer/contracts/maintenance/compliance.rs",
        "src/maintainer/contracts/maintenance/generators.rs",
        "src/maintainer/suites/control_plane/catalog.rs",
        "src/maintainer/suites/control_plane/run.rs",
        "src/maintainer/suites/control_plane/orchestration.rs",
        "src/maintainer/suites/control_plane/ownership.rs",
        "src/maintainer/suites/control_plane/stale_artifacts.rs",
        "src/maintainer/suites/runtime/catalog.rs",
        "src/maintainer/suites/runtime/run.rs",
        "src/maintainer/suites/runtime/config_surface.rs",
        "src/maintainer/suites/runtime/cross_surface.rs",
        "src/maintainer/suites/runtime/install_runtime.rs",
        "src/maintainer/suites/runtime/repl_bridge.rs",
        "src/maintainer/suites/quality/catalog.rs",
        "src/maintainer/suites/quality/run.rs",
        "src/maintainer/suites/quality/command_surface.rs",
        "src/maintainer/suites/quality/release_evidence.rs",
        "src/maintainer/suites/quality/state_laws.rs",
        "src/maintainer/suites/quality/plugin_quality.rs",
        "src/maintainer/suites/resilience/catalog.rs",
        "src/maintainer/suites/resilience/run.rs",
        "src/maintainer/suites/resilience/corruption_campaigns.rs",
        "src/maintainer/suites/resilience/fs_process_adversarial.rs",
        "src/maintainer/suites/resilience/parser_fuzz.rs",
        "src/maintainer/reports/cockpit.rs",
        "src/maintainer/reports/release.rs",
        "src/maintainer/reports/evidence.rs",
        "src/maintainer/reports/config.rs",
        "src/maintainer/reports/python.rs",
        "src/maintainer/reports/runtime_surface.rs",
        "src/maintainer/reports/repository_health.rs",
        "src/maintainer/infra/artifacts.rs",
        "src/maintainer/infra/fs.rs",
        "src/maintainer/infra/process.rs",
        "src/maintainer/infra/clock.rs",
        "src/maintainer/schema/command_registry.rs",
        "src/maintainer/schema/report_envelope.rs",
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
        assert!(!crate_root.join(legacy).exists(), "legacy path must not exist: {legacy}");
    }

    let suites_root = crate_root.join("src/suites");
    let mut files = Vec::<PathBuf>::new();
    collect_files_recursive(&suites_root, &mut files);
    for file in files {
        if file.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let name = file.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        assert!(
            !name.ends_with("_executor.rs") && !name.ends_with("_spec.rs"),
            "legacy suffix file must be removed: {}",
            file.display()
        );
    }
}
