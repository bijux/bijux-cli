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
    let _ = append_control_plane_audit(command_name, status, effect);

    result
}

fn append_control_plane_audit(command_name: &str, status: &str, effect: &str) -> Result<(), String> {
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
