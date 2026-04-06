use crate::commands::DagCli;
use serde::Serialize;
use serde_json::Value;
use std::process::ExitCode;

#[derive(Debug, Serialize)]
pub(crate) struct LintDiagnostic {
    pub code: String,
    pub message: String,
    pub path: String,
    pub hint: Option<String>,
}

#[derive(Debug, Serialize)]
struct JsonEnvelope {
    ok: bool,
    status: String,
    command: String,
    data: Value,
    diagnostics: Vec<Value>,
    error: Option<JsonError>,
}

#[derive(Debug, Serialize)]
struct JsonError {
    category: String,
    code: String,
    message: String,
    exit_code: u8,
}

pub(crate) fn emit_json(
    cli: &DagCli,
    command: &str,
    ok: bool,
    data: Value,
    diagnostics: Vec<Value>,
    code: ExitCode,
) -> Result<ExitCode, ExitCode> {
    if !cli.quiet {
        let exit_value = exit_code_to_u8(code);
        let error = if ok {
            None
        } else {
            let (category, stable_code) = classify_exit(command, exit_value);
            Some(JsonError {
                category: category.to_string(),
                code: stable_code.to_string(),
                message: "command execution failed".to_string(),
                exit_code: exit_value,
            })
        };
        let envelope = JsonEnvelope {
            ok,
            status: if ok { "ok" } else { "invalid" }.to_string(),
            command: command.to_string(),
            data,
            diagnostics,
            error,
        };
        println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
    }
    if ok {
        Ok(code)
    } else {
        Err(code)
    }
}

fn exit_code_to_u8(code: ExitCode) -> u8 {
    if code == ExitCode::SUCCESS {
        0
    } else if code == ExitCode::from(2) {
        2
    } else if code == ExitCode::from(3) {
        3
    } else {
        1
    }
}

fn classify_exit(command: &str, code: u8) -> (&'static str, &'static str) {
    match (command, code) {
        (_, 2) if command.contains("validate") || command.contains("lint") => {
            ("validation", "BJX-VALIDATION-001")
        }
        (_, 2) if command.contains("replay") => ("replay", "BJX-REPLAY-001"),
        (_, 2) if command.contains("cache") => ("cache", "BJX-CACHE-001"),
        (_, 2) => ("compatibility", "BJX-COMPAT-001"),
        (_, 3) => ("io", "BJX-IO-001"),
        _ => ("internal", "BJX-INTERNAL-001"),
    }
}
