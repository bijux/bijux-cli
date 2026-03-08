use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn main() {
    let root = workspace_root();
    let registry_path = root.join("evidence/perf/scenario_registry.json");
    let registry: Value =
        serde_json::from_str(&fs::read_to_string(&registry_path).expect("read scenario registry"))
            .expect("parse registry");

    let mut selected = Vec::new();
    for entry in registry["entries"].as_array().into_iter().flatten() {
        let id = entry["id"].as_str().unwrap_or_default();
        if id.contains("replay") || id.contains("diff") {
            selected.push(entry.clone());
        }
    }

    let report = json!({
        "generated_from": "evidence/perf/scenario_registry.json",
        "focus": "replay-diff",
        "scenario_count": selected.len(),
        "scenarios": selected
    });

    let out = root.join("docs/reports/foundation/replay_diff_benchmark_focus_report.json");
    fs::create_dir_all(out.parent().expect("parent")).expect("mkdir");
    fs::write(&out, serde_json::to_vec_pretty(&report).expect("encode")).expect("write");
}
