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

fn build_json_envelope(
    command: &str,
    ok: bool,
    data: Value,
    diagnostics: Vec<Value>,
    code: ExitCode,
) -> JsonEnvelope {
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
    JsonEnvelope {
        ok,
        status: if ok { "ok" } else { "invalid" }.to_string(),
        command: command.to_string(),
        data,
        diagnostics,
        error,
    }
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
        let envelope = build_json_envelope(command, ok, data, diagnostics, code);
        println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
    }
    if ok {
        Ok(code)
    } else {
        Err(code)
    }
}

pub(crate) fn emit_json_line(
    command: &str,
    ok: bool,
    data: Value,
    diagnostics: Vec<Value>,
    code: ExitCode,
) {
    let envelope = build_json_envelope(command, ok, data, diagnostics, code);
    println!("{}", serde_json::to_string(&envelope).unwrap());
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

#[cfg(test)]
mod tests {
    use super::build_json_envelope;
    use serde_json::json;
    use std::process::ExitCode;

    #[test]
    fn json_envelope_uses_machine_stable_error_metadata() {
        let envelope = build_json_envelope(
            "dag.run",
            false,
            json!({"reason":"failed"}),
            Vec::new(),
            ExitCode::from(3),
        );

        let rendered = serde_json::to_value(&envelope).expect("serialize envelope");
        assert_eq!(rendered["status"], "invalid");
        assert_eq!(rendered["command"], "dag.run");
        assert_eq!(rendered["error"]["category"], "io");
        assert_eq!(rendered["error"]["code"], "BJX-IO-001");
        assert_eq!(rendered["error"]["exit_code"], 3);
    }
}
