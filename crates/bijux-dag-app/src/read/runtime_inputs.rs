use crate::fs_input::read_utf8_file;
use serde_json::{Map, Value};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeInputBinding {
    pub(crate) effective_inputs: Map<String, Value>,
    pub(crate) human_summary: Map<String, Value>,
    pub(crate) redacted_keys: Vec<String>,
}

pub(crate) fn bind_runtime_inputs(
    declared_inputs: &Map<String, Value>,
    inputs_file: Option<&Path>,
    cli_inputs: &[String],
) -> Result<RuntimeInputBinding, String> {
    let mut effective_inputs = declared_inputs.clone();

    if let Some(path) = inputs_file {
        for (key, value) in parse_inputs_file(path)? {
            ensure_declared_input(&key, declared_inputs)?;
            effective_inputs.insert(key, value);
        }
    }

    for assignment in cli_inputs {
        let (key, value) = parse_cli_input_assignment(assignment)?;
        ensure_declared_input(&key, declared_inputs)?;
        effective_inputs.insert(key, value);
    }

    let (human_summary, redacted_keys) = redact_input_summary(&effective_inputs);
    Ok(RuntimeInputBinding { effective_inputs, human_summary, redacted_keys })
}

fn parse_inputs_file(path: &Path) -> Result<Map<String, Value>, String> {
    let payload = read_utf8_file(path)
        .map_err(|error| format!("failed to read inputs file {}: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&payload)
        .map_err(|error| format!("inputs file {} is not valid JSON: {error}", path.display()))?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(format!(
            "inputs file {} must contain a JSON object at the top level",
            path.display()
        )),
    }
}

fn parse_cli_input_assignment(raw: &str) -> Result<(String, Value), String> {
    let Some((key, value)) = raw.split_once('=') else {
        return Err(format!("input assignment must use key=value: {raw}"));
    };
    let key = key.trim();
    if key.is_empty() {
        return Err("input assignment key must not be empty".to_string());
    }
    Ok((key.to_string(), parse_runtime_input_value(value)))
}

fn ensure_declared_input(key: &str, declared_inputs: &Map<String, Value>) -> Result<(), String> {
    if declared_inputs.contains_key(key) {
        return Ok(());
    }
    Err(format!("runtime input is not declared in graph.inputs: {key}"))
}

fn parse_runtime_input_value(raw: &str) -> Value {
    serde_json::from_str::<Value>(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
}

pub(crate) fn is_secret_like_input_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("secret")
        || key.contains("token")
        || key.contains("password")
        || key.contains("api_key")
        || key.contains("apikey")
        || key.contains("credential")
}

fn redact_input_summary(inputs: &Map<String, Value>) -> (Map<String, Value>, Vec<String>) {
    let mut summary = Map::new();
    let mut redacted_keys = Vec::new();
    for (key, value) in inputs {
        if is_secret_like_input_key(key) {
            summary.insert(key.clone(), Value::String("[REDACTED]".to_string()));
            redacted_keys.push(key.clone());
        } else {
            summary.insert(key.clone(), value.clone());
        }
    }
    redacted_keys.sort();
    (summary, redacted_keys)
}

#[cfg(test)]
mod tests {
    use super::{bind_runtime_inputs, is_secret_like_input_key};
    use serde_json::{json, Map, Value};

    fn declared_inputs() -> Map<String, Value> {
        let Value::Object(map) = json!({
            "region": "eu-west-1",
            "attempts": 1,
            "api_token": null
        }) else {
            unreachable!("declared inputs object")
        };
        map
    }

    #[test]
    fn cli_inputs_override_declared_defaults() {
        let binding = bind_runtime_inputs(
            &declared_inputs(),
            None,
            &["region=us-east-1".to_string(), "attempts=3".to_string()],
        )
        .expect("binding");
        assert_eq!(binding.effective_inputs["region"], "us-east-1");
        assert_eq!(binding.effective_inputs["attempts"], 3);
    }

    #[test]
    fn cli_inputs_accept_json_literals_and_plain_strings() {
        let binding = bind_runtime_inputs(
            &declared_inputs(),
            None,
            &["region=plain-text".to_string(), "attempts=2".to_string()],
        )
        .expect("binding");
        assert_eq!(binding.effective_inputs["region"], "plain-text");
        assert_eq!(binding.effective_inputs["attempts"], 2);
    }

    #[test]
    fn undeclared_runtime_input_is_rejected() {
        let error =
            bind_runtime_inputs(&declared_inputs(), None, &["unknown=x".to_string()]).unwrap_err();
        assert!(error.contains("runtime input is not declared"));
    }

    #[test]
    fn secret_like_keys_are_redacted_in_human_summary() {
        let binding = bind_runtime_inputs(
            &declared_inputs(),
            None,
            &["api_token=s3cr3t".to_string()],
        )
        .expect("binding");
        assert_eq!(binding.effective_inputs["api_token"], "s3cr3t");
        assert_eq!(binding.human_summary["api_token"], "[REDACTED]");
        assert_eq!(binding.redacted_keys, vec!["api_token".to_string()]);
        assert!(is_secret_like_input_key("DB_PASSWORD"));
    }
}
