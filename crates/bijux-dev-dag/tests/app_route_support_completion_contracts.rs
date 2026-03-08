use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn app_route_support_completion_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/app_route_support_completion_report.md",
        "configs/suites/app_route_support_fast.json",
        "crates/bijux-dev-dag/tests/app_route_support_fast_suite_contracts.rs",
        "crates/bijux-dag-app/tests/route_output_wording_snapshot_contracts.rs",
        "crates/bijux-dag-app/tests/snapshots/route_concise_wording.txt",
        "crates/bijux-dag-app/tests/snapshots/route_detailed_wording.txt",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing app route-support completion artifact {rel}"
        );
    }

    let report = fs::read_to_string(
        root.join("docs/reports/foundation/app_route_support_completion_report.md"),
    )
    .expect("read report");
    for required in [
        "(421-440)",
        "routes/output_selection.rs",
        "routes/response.rs",
        "routes/run_lookup.rs",
        "app_route_support_fast.json",
    ] {
        assert!(
            report.contains(required),
            "app route-support completion report missing {required}"
        );
    }
}
