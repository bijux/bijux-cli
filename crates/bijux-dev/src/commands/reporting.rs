use crate::commands::model::{CommandContext, CommandEffect};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;

pub(crate) fn run_command_reported<F>(
    context: &CommandContext,
    command: &str,
    effect: CommandEffect,
    data: Value,
    run: F,
) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    run_text_or_json_report(context, command, command, effect.label(), data, run, true)
}

pub(crate) fn run_text_or_json_report(
    context: &CommandContext,
    command: &str,
    command_name: &str,
    effect: &str,
    data: Value,
    run: impl FnOnce() -> Result<(), String>,
    include_data_on_success: bool,
) -> Result<(), String> {
    let result = run();
    let (status, error) = match &result {
        Ok(_) => ("ok", None),
        Err(err) => ("error", Some(err.clone())),
    };

    let mut report = json!({
        "command": command_name,
        "status": status,
        "effect": effect,
        "data": data,
    });
    if let Some(error) = error {
        report["error"] = Value::String(error);
    }

    if context.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("json print")
        );
    } else if include_data_on_success || status == "error" {
        let value = report.to_string();
        println!("[{command}] {status} ({effect}): {value}",);
    } else {
        println!("[{command}] {status} ({effect})");
    }

    if let Some(report_path) = context.report.as_ref() {
        let output = serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?;
        fs::write(report_path, output).map_err(|err| err.to_string())?;
    }
    if std::env::var("BIJUX_DEV_DAG_DISABLE_AUDIT_APPEND")
        .ok()
        .as_deref()
        != Some("1")
    {
        let _ = append_control_plane_audit(command_name, status, effect);
    }

    result
}

fn append_control_plane_audit(
    command_name: &str,
    status: &str,
    effect: &str,
) -> Result<(), String> {
    let root = crate::commands::repo_root()?;
    let audit_dir = root.join("artifacts").join("reports");
    fs::create_dir_all(&audit_dir).map_err(|err| err.to_string())?;
    let audit_path = audit_dir.join("control-plane-audit.jsonl");
    let event = json!({
        "action": command_name,
        "status": status,
        "effect": effect,
        "ts_unix_ms": crate::commands::now_millis(),
    });
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_path)
        .map_err(|err| err.to_string())?;
    writeln!(file, "{event}").map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::run_text_or_json_report;
    use crate::commands::model::CommandContext;
    use serde_json::Value;

    #[test]
    fn report_shape_is_stable_for_success_case() {
        let temp = tempfile::tempdir().expect("tmp");
        let report_path = temp.path().join("report.json");
        let context = CommandContext {
            json: false,
            report: Some(report_path.clone()),
        };
        std::env::set_var("BIJUX_DEV_DAG_DISABLE_AUDIT_APPEND", "1");
        run_text_or_json_report(
            &context,
            "test.command",
            "test.command",
            "validation",
            serde_json::json!({"sample": true}),
            || Ok(()),
            true,
        )
        .expect("report run");
        let payload = std::fs::read_to_string(&report_path).expect("read report");
        let value: Value = serde_json::from_str(&payload).expect("parse report");
        assert_eq!(value["command"], "test.command");
        assert_eq!(value["status"], "ok");
        assert_eq!(value["effect"], "validation");
        assert!(value.get("data").is_some());
        std::env::remove_var("BIJUX_DEV_DAG_DISABLE_AUDIT_APPEND");
    }
}
