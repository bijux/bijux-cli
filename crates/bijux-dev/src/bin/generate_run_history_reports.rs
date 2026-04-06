use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn required_fields(schema_path: &Path) -> Vec<String> {
    let payload = fs::read_to_string(schema_path).expect("schema read");
    let schema: Value = serde_json::from_str(&payload).expect("schema parse");
    schema
        .get("required")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default()
}

fn main() {
    let root = workspace_root();
    let operator = root.join("configs/dag/schema/operator");

    let mut required = BTreeMap::<String, Vec<String>>::new();
    for name in [
        "run_summary.schema.json",
        "run_show.schema.json",
        "run_inspect.schema.json",
        "run_history.schema.json",
    ] {
        required.insert(name.to_string(), required_fields(&operator.join(name)));
    }

    let report = json!({
        "generated_from": "configs/dag/schema/operator",
        "surfaces": required,
        "lineage_boundary_doc": "docs/spec/RUN_VS_ARTIFACT_LINEAGE.md"
    });

    let out = root.join("docs/reports/foundation/run_history_api_report.json");
    fs::create_dir_all(out.parent().expect("report parent")).expect("mkdir report parent");
    fs::write(&out, serde_json::to_vec_pretty(&report).expect("encode report"))
        .expect("write report");
}
