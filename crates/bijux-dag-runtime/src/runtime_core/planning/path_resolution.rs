use bijux_dag_artifacts::{is_normalized_relative_path, RunDirLayout};
use bijux_dag_core::is_known_path_variable;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

const CONTAINER_INPUTS_DIR: &str = "/bijux/node/inputs";
const CONTAINER_OUTPUTS_DIR: &str = "/bijux/node/outputs";
const CONTAINER_WORK_DIR: &str = "/bijux/node/work";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AbsolutePathPolicy {
    #[default]
    AllowLiteral,
    #[serde(alias = "deny")]
    DenyLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePathBindings {
    pub run_dir: Option<String>,
    pub work_dir: String,
    pub inputs_dir: String,
    pub outputs_dir: String,
    pub cache_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathBindingSurface {
    Host,
    Container,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathExpression<'a> {
    variable: &'a str,
    relative_path: Option<&'a str>,
}

impl NodePathBindings {
    pub fn for_host(layout: &RunDirLayout, node_id: &str, cache_dir: Option<&Path>) -> Self {
        Self {
            run_dir: Some(layout.staging_path.display().to_string()),
            work_dir: layout.node_work_dir(node_id).display().to_string(),
            inputs_dir: layout.node_inputs_dir(node_id).display().to_string(),
            outputs_dir: layout.node_outputs_dir(node_id).display().to_string(),
            cache_dir: cache_dir.map(|path| path.display().to_string()),
        }
    }

    pub fn for_container() -> Self {
        Self {
            run_dir: None,
            work_dir: CONTAINER_WORK_DIR.to_string(),
            inputs_dir: CONTAINER_INPUTS_DIR.to_string(),
            outputs_dir: CONTAINER_OUTPUTS_DIR.to_string(),
            cache_dir: None,
        }
    }

    fn variable_value(&self, name: &str) -> Option<&str> {
        match name {
            "run_dir" => self.run_dir.as_deref(),
            "work_dir" => Some(&self.work_dir),
            "inputs_dir" => Some(&self.inputs_dir),
            "outputs_dir" => Some(&self.outputs_dir),
            "cache_dir" => self.cache_dir.as_deref(),
            _ => None,
        }
    }
}

pub(crate) fn bind_path_variables_in_value(
    value: &Value,
    bindings: &NodePathBindings,
) -> Result<Value, String> {
    match value {
        Value::String(text) => {
            if let Some(resolved) = resolve_path_expression(text, bindings)? {
                Ok(Value::String(resolved))
            } else {
                Ok(Value::String(text.clone()))
            }
        }
        Value::Array(items) => {
            let mut resolved = Vec::with_capacity(items.len());
            for item in items {
                resolved.push(bind_path_variables_in_value(item, bindings)?);
            }
            Ok(Value::Array(resolved))
        }
        Value::Object(map) => {
            let mut resolved = serde_json::Map::new();
            for (key, entry) in map {
                resolved.insert(key.clone(), bind_path_variables_in_value(entry, bindings)?);
            }
            Ok(Value::Object(resolved))
        }
        literal => Ok(literal.clone()),
    }
}

pub(crate) fn resolve_container_argv(
    argv: &[String],
    bindings: &NodePathBindings,
) -> Result<Vec<String>, String> {
    let mut resolved = Vec::with_capacity(argv.len());
    for entry in argv {
        if let Some(value) = resolve_path_expression(entry, bindings)? {
            resolved.push(value);
        } else {
            resolved.push(entry.clone());
        }
    }
    Ok(resolved)
}

pub(crate) fn resolve_container_workdir(
    workdir: Option<&str>,
    bindings: &NodePathBindings,
    absolute_path_policy: AbsolutePathPolicy,
) -> Result<String, String> {
    let Some(workdir) = workdir else {
        return Ok(bindings.work_dir.clone());
    };
    if let Some(resolved) = resolve_path_expression(workdir, bindings)? {
        return Ok(resolved);
    }
    if workdir.starts_with('/') {
        return match absolute_path_policy {
            AbsolutePathPolicy::AllowLiteral => Ok(workdir.to_string()),
            AbsolutePathPolicy::DenyLiteral => {
                Err(format!("literal absolute workdir is denied by policy: {workdir}"))
            }
        };
    }
    if !is_normalized_relative_path(workdir) {
        return Err(format!("invalid relative workdir: {workdir}"));
    }
    Ok(format!("{}/{}", bindings.work_dir, workdir))
}

fn resolve_path_expression(
    value: &str,
    bindings: &NodePathBindings,
) -> Result<Option<String>, String> {
    let Some(expression) = parse_path_expression(value)? else {
        return Ok(None);
    };
    let base = bindings.variable_value(expression.variable).ok_or_else(|| {
        format!("path variable unavailable for this execution surface: {}", expression.variable)
    })?;
    match expression.relative_path {
        Some(relative_path) => Ok(Some(format!("{base}/{relative_path}"))),
        None => Ok(Some(base.to_string())),
    }
}

fn parse_path_expression(value: &str) -> Result<Option<PathExpression<'_>>, String> {
    if !value.starts_with('{') {
        return Ok(None);
    }
    let Some(close_index) = value.find('}') else {
        return Err(format!("invalid path variable expression: {value}"));
    };
    let variable = &value[1..close_index];
    if variable.is_empty() || !is_known_path_variable(variable) {
        return Err(format!("unknown path variable expression: {value}"));
    }
    let rest = &value[(close_index + 1)..];
    if rest.is_empty() {
        return Ok(Some(PathExpression { variable, relative_path: None }));
    }
    let Some(relative_path) = rest.strip_prefix('/') else {
        return Err(format!("invalid path variable expression: {value}"));
    };
    if !is_normalized_relative_path(relative_path) {
        return Err(format!("invalid path variable suffix: {relative_path}"));
    }
    Ok(Some(PathExpression { variable, relative_path: Some(relative_path) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_bindings_resolve_path_expressions_recursively() {
        let dir = tempfile::tempdir().expect("tmp");
        let layout = RunDirLayout::preview(dir.path(), Some("paths")).expect("layout");
        let bindings = NodePathBindings::for_host(&layout, "node", Some(dir.path().join("cache").as_path()));
        let value = serde_json::json!({
            "argv": ["cp", "{inputs_dir}/seed.txt", "{outputs_dir}/value.txt"],
            "nested": {"target": "{cache_dir}/reuse.json"}
        });
        let resolved = bind_path_variables_in_value(&value, &bindings).expect("resolve");
        assert_eq!(
            resolved["argv"][1].as_str(),
            Some(layout.node_inputs_dir("node").join("seed.txt").display().to_string().as_str())
        );
        assert_eq!(
            resolved["nested"]["target"].as_str(),
            Some(dir.path().join("cache").join("reuse.json").display().to_string().as_str())
        );
    }

    #[test]
    fn container_workdir_rejects_denied_absolute_literals() {
        let err = resolve_container_workdir(
            Some("/workspace"),
            &NodePathBindings::for_container(),
            AbsolutePathPolicy::DenyLiteral,
        )
        .expect_err("absolute path must be denied");
        assert!(err.contains("literal absolute workdir"));
    }

    #[test]
    fn container_workdir_resolves_relative_and_variable_paths() {
        let bindings = NodePathBindings::for_container();
        assert_eq!(
            resolve_container_workdir(
                Some("scratch"),
                &bindings,
                AbsolutePathPolicy::DenyLiteral,
            )
            .expect("relative workdir"),
            "/bijux/node/work/scratch"
        );
        assert_eq!(
            resolve_container_workdir(
                Some("{outputs_dir}/materialized"),
                &bindings,
                AbsolutePathPolicy::DenyLiteral,
            )
            .expect("variable workdir"),
            "/bijux/node/outputs/materialized"
        );
    }
}
