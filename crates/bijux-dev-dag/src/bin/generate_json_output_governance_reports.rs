use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn write_file(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, body).map_err(|e| e.to_string())
}

fn schema_to_example_dir(schema_rel: &str) -> String {
    let stem = Path::new(schema_rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("schema");
    format!("evidence/operator/examples/stable_json/{stem}")
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
}

fn main() -> Result<(), String> {
    let root = repo_root();
    let policy_raw = fs::read_to_string(root.join("configs/policy/json_output_governance.json"))
        .map_err(|e| e.to_string())?;
    let policy: serde_json::Value = serde_json::from_str(&policy_raw).map_err(|e| e.to_string())?;

    let out_dir = root.join("docs/reports/foundation");
    let ref_dir = root.join("docs/reference");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&ref_dir).map_err(|e| e.to_string())?;

    let families = policy["stable_command_families"]
        .as_array()
        .ok_or_else(|| "stable_command_families must be array".to_string())?;

    let mut all_schemas: BTreeSet<String> = BTreeSet::new();
    for fam in families {
        for schema in fam["schemas"].as_array().cloned().unwrap_or_default() {
            if let Some(s) = schema.as_str() {
                all_schemas.insert(s.to_string());
            }
        }
    }

    for schema in &all_schemas {
        let schema_path = root.join(schema);
        let schema_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&schema_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let example_dir = root.join(schema_to_example_dir(schema));
        fs::create_dir_all(&example_dir).map_err(|e| e.to_string())?;

        let mut minimal_data = serde_json::Map::new();
        for req in schema_json["required"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            if let Some(k) = req.as_str() {
                minimal_data.insert(
                    k.to_string(),
                    serde_json::Value::String("example".to_string()),
                );
            }
        }
        let minimal =
            serde_json::json!({"schema": schema, "example_type":"minimal", "data": minimal_data});

        let mut maximal_data = serde_json::Map::new();
        if let Some(props) = schema_json["properties"].as_object() {
            for k in props.keys() {
                maximal_data.insert(k.clone(), serde_json::Value::String("example".to_string()));
            }
        }
        let maximal =
            serde_json::json!({"schema": schema, "example_type":"maximal", "data": maximal_data});

        fs::write(
            example_dir.join("minimal.json"),
            serde_json::to_vec_pretty(&minimal).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            example_dir.join("maximal.json"),
            serde_json::to_vec_pretty(&maximal).unwrap(),
        )
        .map_err(|e| e.to_string())?;
    }

    let mut inv_a = String::from("# JSON Command to Schema Inventory Report\n\n| Family | Command | Schema |\n| --- | --- | --- |\n");
    for fam in families {
        let family = fam["family"].as_str().unwrap_or("");
        let commands = fam["commands"].as_array().cloned().unwrap_or_default();
        let schemas = fam["schemas"].as_array().cloned().unwrap_or_default();
        for cmd in &commands {
            for schema in &schemas {
                inv_a.push_str(&format!(
                    "| `{family}` | `{}` | `{}` |\n",
                    cmd.as_str().unwrap_or(""),
                    schema.as_str().unwrap_or("")
                ));
            }
        }
    }
    write_file(
        &out_dir.join("JSON_COMMAND_SCHEMA_INVENTORY_REPORT.md"),
        &inv_a,
    )?;

    let mut inv_b = String::from("# Schema to Command and Test Inventory Report\n\n| Schema | Family | Commands | Lockstep markers |\n| --- | --- | --- | --- |\n");
    for fam in families {
        let family = fam["family"].as_str().unwrap_or("");
        let commands = fam["commands"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        let markers = fam["lockstep_markers"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        for schema in fam["schemas"].as_array().cloned().unwrap_or_default() {
            inv_b.push_str(&format!(
                "| `{}` | `{family}` | `{commands}` | `{markers}` |\n",
                schema.as_str().unwrap_or("")
            ));
        }
    }
    write_file(
        &out_dir.join("SCHEMA_COMMAND_TEST_INVENTORY_REPORT.md"),
        &inv_b,
    )?;

    let mut missing_examples = String::from("# Schemas Without Example Output Report\n\n| Schema | Missing minimal | Missing maximal |\n| --- | --- | --- |\n");
    let mut missing_example_count = 0usize;
    for schema in &all_schemas {
        let dir = root.join(schema_to_example_dir(schema));
        let min_missing = !dir.join("minimal.json").exists();
        let max_missing = !dir.join("maximal.json").exists();
        if min_missing || max_missing {
            missing_examples.push_str(&format!(
                "| `{schema}` | `{min_missing}` | `{max_missing}` |\n"
            ));
            missing_example_count += 1;
        }
    }
    missing_examples.push_str(&format!(
        "\nMissing schema examples: {missing_example_count}\n"
    ));
    write_file(
        &out_dir.join("schema_without_example_output_report.md"),
        &missing_examples,
    )?;

    let mut rs_files = Vec::new();
    collect_rs_files(&root.join("crates/bijux-dag-app/tests"), &mut rs_files);
    collect_rs_files(&root.join("crates/bijux-dev-dag/tests"), &mut rs_files);
    let mut test_sources = String::new();
    for file in rs_files {
        if let Ok(text) = fs::read_to_string(file) {
            test_sources.push_str(&text);
            test_sources.push('\n');
        }
    }

    let mut missing_lock = String::from("# Commands Without JSON Lockstep Report\n\n| Family | Command | Missing lockstep marker |\n| --- | --- | --- |\n");
    let mut missing_lock_count = 0usize;
    for fam in families {
        let family = fam["family"].as_str().unwrap_or("");
        let commands = fam["commands"].as_array().cloned().unwrap_or_default();
        let markers = fam["lockstep_markers"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for (idx, cmd) in commands.iter().enumerate() {
            let marker = markers.get(idx).and_then(|v| v.as_str()).unwrap_or("");
            if marker.is_empty() || !test_sources.contains(marker) {
                missing_lock.push_str(&format!(
                    "| `{family}` | `{}` | `{marker}` |\n",
                    cmd.as_str().unwrap_or("")
                ));
                missing_lock_count += 1;
            }
        }
    }
    missing_lock.push_str(&format!(
        "\nCommands missing lockstep tests: {missing_lock_count}\n"
    ));
    write_file(
        &out_dir.join("commands_without_json_lockstep_report.md"),
        &missing_lock,
    )?;

    let mut schema_registry = String::from("# Schema Registry\n\nGenerated from `configs/policy/json_output_governance.json`.\n\n| Schema | Example directory |\n| --- | --- |\n");
    for schema in &all_schemas {
        schema_registry.push_str(&format!(
            "| `{schema}` | `{}` |\n",
            schema_to_example_dir(schema)
        ));
    }
    write_file(&ref_dir.join("SCHEMA_REGISTRY.md"), &schema_registry)?;

    let mut command_registry = String::from("# Stable JSON Output Command Registry\n\nGenerated from `configs/policy/json_output_governance.json`.\n\n| Family | Commands |\n| --- | --- |\n");
    for fam in families {
        let family = fam["family"].as_str().unwrap_or("");
        let commands = fam["commands"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        command_registry.push_str(&format!("| `{family}` | `{commands}` |\n"));
    }
    write_file(
        &ref_dir.join("STABLE_JSON_OUTPUT_COMMAND_REGISTRY.md"),
        &command_registry,
    )?;

    Ok(())
}
