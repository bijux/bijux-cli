#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-MAINTAINER-CONTROL-PLANE-REPORTS" => {
            let generated_at = generated_at_utc();
            let required_commands = vec![
                "dev cli status",
                "dev cli parity",
                "dev cli route-audit",
                "dev cli state-audit",
                "dev cli maintenance-audit",
                "dev cli crate-health",
                "dev cli package-health",
                "dev cli docs-audit",
            ];
            let replacements: BTreeMap<&str, &str> = BTreeMap::new();
            let command_samples = fs::read_to_string(
                workspace_root.join("artifacts/status/dev_cli_control_plane_samples.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let mut inventory = Vec::<Value>::new();
            for path in collect_files(&workspace_root.join("maintenance")) {
                let relp = rel(&path, workspace_root);
                if relp.starts_with("maintenance/obsolete-status/") {
                    continue;
                }
                let replacement = replacements.get(relp.as_str()).copied().unwrap_or("");
                inventory.push(json!({"path":relp,"replacement_command":replacement,"status":if replacement.is_empty(){"remaining"}else{"replaced"}}));
            }
            inventory.sort_by(|a, b| {
                a.get("path")
                    .and_then(Value::as_str)
                    .cmp(&b.get("path").and_then(Value::as_str))
            });
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_maintenance_outside_dev_cli.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","maintenance":inventory,"summary":{"total":inventory.len(),"replaced":inventory.iter().filter(|r| r["status"]=="replaced").count(),"remaining":inventory.iter().filter(|r| r["status"]=="remaining").count()}})).ok()?;
            let commands = required_commands.iter().map(|command| {
                                let sample = command_samples.get(*command).cloned().unwrap_or_else(|| json!({}));
                                json!({"command":command,"json_sample_present":sample.get("json").is_some(),"text_sample_present":sample.get("text").is_some(),"json_top_level_keys":sample.get("json_top_level_keys").cloned().unwrap_or_else(|| json!([]))})
                            }).collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_control_plane_commands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","required_commands":required_commands,"commands":commands})).ok()?;
            let mut text =
                format!("Maintainer control plane summary\nGenerated at: {generated_at}\n\n");
            for row in &commands {
                let keys = row
                    .get("json_top_level_keys")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
                    .join(", ");
                text.push_str(&format!(
                    "- {}: json_keys={}\n",
                    row.get("command").and_then(Value::as_str).unwrap_or(""),
                    if keys.is_empty() { "(none)" } else { &keys }
                ));
            }
            text.push_str("\nDefault maintainer command: bijux dev cli status\nPolicy: use dev cli command surfaces before creating new ad-hoc maintenance.\n");
            fs::write(
                workspace_root.join("artifacts/status/maintainer_control_plane_text_report.txt"),
                text,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_control_plane_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","maintenance_outside_dev_cli":fs::read_to_string(workspace_root.join("artifacts/status/maintainer_maintenance_outside_dev_cli.json")).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_else(|| json!({})),"commands":fs::read_to_string(workspace_root.join("artifacts/status/maintainer_control_plane_commands.json")).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_else(|| json!({})),"text_report":"artifacts/status/maintainer_control_plane_text_report.txt"})).ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                    "artifacts/status/maintainer_maintenance_outside_dev_cli.json",
                    "artifacts/status/maintainer_control_plane_commands.json",
                    "artifacts/status/maintainer_control_plane_text_report.txt",
                    "artifacts/status/maintainer_control_plane_report.json"
                ]}),
            )
        }
        _ => None,
    }
}
