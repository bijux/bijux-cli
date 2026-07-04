use crate::fs_input::read_utf8_file;
use bijux_dag_core::{materialize_graph_input_value, Graph, GraphInputSpec, ParamValue};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeInputBinding {
    pub(crate) bound_inputs: BTreeMap<String, GraphInputSpec>,
    pub(crate) effective_inputs: BTreeMap<String, Value>,
    pub(crate) human_summary: Map<String, Value>,
    pub(crate) redacted_keys: Vec<String>,
}

pub(crate) fn bind_runtime_inputs(
    declared_inputs: &BTreeMap<String, GraphInputSpec>,
    inputs_file: Option<&Path>,
    cli_inputs: &[String],
) -> Result<RuntimeInputBinding, String> {
    let mut bound_inputs = declared_inputs.clone();

    if let Some(path) = inputs_file {
        for (key, value) in parse_inputs_file(path)? {
            bind_runtime_input_value(&key, value, declared_inputs, &mut bound_inputs)?;
        }
    }

    for assignment in cli_inputs {
        let (key, value) = parse_cli_input_assignment(assignment)?;
        bind_runtime_input_value(&key, value, declared_inputs, &mut bound_inputs)?;
    }

    let effective_inputs = effective_inputs_from_specs(&bound_inputs)?;
    let (human_summary, redacted_keys) = redact_input_summary(&effective_inputs);
    Ok(RuntimeInputBinding { bound_inputs, effective_inputs, human_summary, redacted_keys })
}

pub(crate) fn missing_required_graph_inputs(graph: &Graph) -> Vec<String> {
    let mut required = BTreeSet::new();
    for node in &graph.nodes {
        collect_required_graph_inputs(&node.params, &mut required);
    }

    required
        .into_iter()
        .filter(|key| graph.inputs.get(key).and_then(GraphInputSpec::effective_value).is_none())
        .collect()
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

fn bind_runtime_input_value(
    key: &str,
    value: Value,
    declared_inputs: &BTreeMap<String, GraphInputSpec>,
    bound_inputs: &mut BTreeMap<String, GraphInputSpec>,
) -> Result<(), String> {
    let Some(spec) = declared_inputs.get(key) else {
        return Err(format!("runtime input is not declared in graph.inputs: {key}"));
    };
    let normalized = materialize_graph_input_value(spec, &value, &format!("/inputs/{key}"))
        .map_err(|error| format!("runtime input at {}: {}", error.path, error.message))?;
    bound_inputs.insert(key.to_string(), spec.with_effective_value(normalized));
    Ok(())
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

fn redact_input_summary(inputs: &BTreeMap<String, Value>) -> (Map<String, Value>, Vec<String>) {
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

fn effective_inputs_from_specs(
    inputs: &BTreeMap<String, GraphInputSpec>,
) -> Result<BTreeMap<String, Value>, String> {
    let mut effective = BTreeMap::new();
    for (key, spec) in inputs {
        if let Some(value) = spec.effective_value() {
            let materialized =
                materialize_graph_input_value(spec, value, &format!("/inputs/{key}")).map_err(
                    |error| format!("runtime input at {}: {}", error.path, error.message),
                )?;
            effective.insert(key.clone(), materialized);
        }
    }
    Ok(effective)
}

fn collect_required_graph_inputs(value: &ParamValue, required: &mut BTreeSet<String>) {
    match value {
        ParamValue::Ref(reference) => {
            if let Some(input_name) = &reference.graph_input {
                required.insert(input_name.clone());
            }
        }
        ParamValue::Array(items) => {
            for item in items {
                collect_required_graph_inputs(item, required);
            }
        }
        ParamValue::Object(map) => {
            for item in map.values() {
                collect_required_graph_inputs(item, required);
            }
        }
        ParamValue::Literal(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{bind_runtime_inputs, is_secret_like_input_key, missing_required_graph_inputs};
    use bijux_dag_core::{parse_graph_strict, GraphInputSpec};
    use serde_json::json;
    use std::collections::BTreeMap;

    fn declared_inputs() -> BTreeMap<String, GraphInputSpec> {
        serde_json::from_value(json!({
            "region": {"type":"string","default":"eu-west-1"},
            "attempts": {"type":"integer","default":1},
            "api_token": {"type":"string","required":true}
        }))
        .expect("declared inputs")
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
        let binding =
            bind_runtime_inputs(&declared_inputs(), None, &["api_token=s3cr3t".to_string()])
                .expect("binding");
        assert_eq!(binding.effective_inputs["api_token"], "s3cr3t");
        assert_eq!(binding.human_summary["api_token"], "[REDACTED]");
        assert_eq!(binding.redacted_keys, vec!["api_token".to_string()]);
        assert!(is_secret_like_input_key("DB_PASSWORD"));
    }

    #[test]
    fn missing_required_graph_inputs_only_flags_referenced_null_values() {
        let graph = parse_graph_strict(
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":[],"tags":[]},
              "inputs":{
                "region":{"type":"string","required":true},
                "api_token":{"type":"string","required":true},
                "unused":{"type":"string","required":true}
              },
              "nodes":[
                {
                  "id":"n1",
                  "kind":"const",
                  "inputs":[],
                  "outputs":[{"name":"value","path":"out.json"}],
                  "params":{
                    "value":{
                      "region":{"graph_input":"region"},
                      "token":{"graph_input":"api_token"}
                    }
                  }
                }
              ],
              "edges":[]
            }"#,
        )
        .expect("graph");

        assert_eq!(
            missing_required_graph_inputs(&graph),
            vec!["api_token".to_string(), "region".to_string()]
        );
    }
}
