use crate::canonical::{error, is_valid_canonical_name, is_valid_output_path, severity_rank, warn};
use crate::expansion::{expand_graph, expansion_error_diagnostic};
use crate::{
    is_known_path_variable, materialize_graph_input_value, EdgeKind, Effect, Graph, GraphError,
    GraphInputKind, GraphInputSpec, Node, ParamValue, SemanticNodeKind, Severity, TriggerRule,
    ValidationDiagnostic,
};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationDomain {
    Schema,
    Semantic,
    Topology,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRule {
    pub id: &'static str,
    pub severity: Severity,
    pub domain: ValidationDomain,
}

const VALIDATION_RULES: &[ValidationRule] = &[
    ValidationRule { id: "E1001", severity: Severity::Error, domain: ValidationDomain::Schema },
    ValidationRule { id: "E1002", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1003", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1004", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1005", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1006", severity: Severity::Error, domain: ValidationDomain::Schema },
    ValidationRule { id: "E1007", severity: Severity::Error, domain: ValidationDomain::Schema },
    ValidationRule { id: "E1008", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1009", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1010", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1011", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1013", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1020", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1021", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1022", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1023", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1024", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1025", severity: Severity::Error, domain: ValidationDomain::Schema },
    ValidationRule { id: "E1026", severity: Severity::Error, domain: ValidationDomain::Schema },
    ValidationRule { id: "E1027", severity: Severity::Error, domain: ValidationDomain::Schema },
    ValidationRule { id: "E1028", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1029", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1030", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1031", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1032", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1033", severity: Severity::Error, domain: ValidationDomain::Schema },
    ValidationRule { id: "E1034", severity: Severity::Error, domain: ValidationDomain::Schema },
    ValidationRule { id: "E1035", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1036", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1037", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1038", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1039", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1040", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1041", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1042", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1043", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1044", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1045", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1046", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1047", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1048", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1049", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1050", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1051", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1052", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1053", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1054", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1055", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1056", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1057", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1058", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1059", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1060", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1061", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1062", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1063", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1064", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1065", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1066", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1067", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1068", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "W2001", severity: Severity::Warning, domain: ValidationDomain::Topology },
    ValidationRule { id: "W2002", severity: Severity::Warning, domain: ValidationDomain::Topology },
];

pub fn validation_rule_registry() -> &'static [ValidationRule] {
    VALIDATION_RULES
}

fn validation_rule(code: &'static str) -> &'static ValidationRule {
    VALIDATION_RULES
        .iter()
        .find(|rule| rule.id == code)
        .unwrap_or_else(|| panic!("missing validation rule registration for {code}"))
}

fn emit_rule(
    diagnostics: &mut Vec<ValidationDiagnostic>,
    code: &'static str,
    message: String,
    path: String,
    hint: Option<String>,
) {
    let diagnostic = match validation_rule(code).severity {
        Severity::Error => error(code, message, path, hint),
        Severity::Warning => warn(code, message, path, hint),
    };
    diagnostics.push(diagnostic);
}

fn valid_env_allowlist_pattern(pattern: &str) -> bool {
    let core = pattern.strip_suffix('*').unwrap_or(pattern);
    if core.is_empty() || pattern == "*" {
        return false;
    }
    core.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn node_param_object(node: &Node) -> Option<&std::collections::BTreeMap<String, ParamValue>> {
    match &node.params {
        ParamValue::Object(map) => Some(map),
        _ => None,
    }
}

fn node_param_literal_string<'a>(node: &'a Node, key: &str) -> Option<&'a str> {
    node_param_object(node).and_then(|params| params.get(key)).and_then(|value| match value {
        ParamValue::Literal(serde_json::Value::String(text)) => Some(text.as_str()),
        _ => None,
    })
}

fn param_value_literal_string(value: &ParamValue) -> Option<&str> {
    match value {
        ParamValue::Literal(serde_json::Value::String(text)) => Some(text.as_str()),
        _ => None,
    }
}

fn node_param_object_field<'a>(
    node: &'a Node,
    key: &str,
) -> Option<&'a std::collections::BTreeMap<String, ParamValue>> {
    node_param_object(node).and_then(|params| params.get(key)).and_then(|value| match value {
        ParamValue::Object(map) => Some(map),
        _ => None,
    })
}

fn node_param_array_field<'a>(node: &'a Node, key: &str) -> Option<&'a [ParamValue]> {
    node_param_object(node).and_then(|params| params.get(key)).and_then(|value| match value {
        ParamValue::Array(items) => Some(items.as_slice()),
        _ => None,
    })
}

fn param_value_is_literal_string(value: &ParamValue) -> bool {
    matches!(value, ParamValue::Literal(serde_json::Value::String(_)))
}

fn param_value_literal_u64(value: &ParamValue) -> Option<u64> {
    match value {
        ParamValue::Literal(serde_json::Value::Number(number)) => number.as_u64(),
        _ => None,
    }
}

fn param_value_literal_bool(value: &ParamValue) -> Option<bool> {
    match value {
        ParamValue::Literal(serde_json::Value::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn is_normalized_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn trigger_rule_supports_conditional_incoming(rule: &TriggerRule) -> bool {
    matches!(rule, TriggerRule::AnySuccess | TriggerRule::AllDone)
}

fn validate_path_variable_expression(value: &str) -> Result<(), String> {
    let Some(close_index) = value.find('}') else {
        return Err(format!("invalid path variable expression: {value}"));
    };
    let variable = &value[1..close_index];
    if variable.is_empty() || !is_known_path_variable(variable) {
        return Err(format!("unknown path variable expression: {value}"));
    }
    let rest = &value[(close_index + 1)..];
    if rest.is_empty() {
        return Ok(());
    }
    let Some(relative_path) = rest.strip_prefix('/') else {
        return Err(format!("invalid path variable expression: {value}"));
    };
    if !is_normalized_relative_path(relative_path) {
        return Err(format!("invalid path variable suffix: {relative_path}"));
    }
    Ok(())
}

fn validate_container_workdir_value(workdir: &str) -> Result<(), String> {
    if workdir.starts_with('{') {
        return validate_path_variable_expression(workdir);
    }
    if workdir.starts_with('/') {
        return Ok(());
    }
    if is_normalized_relative_path(workdir) {
        return Ok(());
    }
    Err(format!("invalid relative workdir: {workdir}"))
}

impl Graph {
    pub fn validate_with_warnings(&self) -> Vec<ValidationDiagnostic> {
        let expanded = match expand_graph(self) {
            Ok(graph) => graph,
            Err(error) => return vec![expansion_error_diagnostic(error)],
        };
        expanded.validate_expanded_with_warnings()
    }

    fn validate_expanded_with_warnings(&self) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();

        validate_graph_inputs(self, &mut diagnostics);

        let mut ids = BTreeSet::new();
        let mut node_map: HashMap<&str, &Node> = HashMap::new();
        let mut branch_output_by_node: HashMap<&str, &str> = HashMap::new();
        let mut branch_decisions_by_node: HashMap<&str, BTreeSet<String>> = HashMap::new();
        let mut dynamic_nodes = BTreeSet::new();
        for node in &self.nodes {
            if !ids.insert(node.id.as_str()) {
                emit_rule(
                    &mut diagnostics,
                    "E1001",
                    format!("duplicate node id: {}", node.id),
                    format!("/nodes/{}", node.id),
                    Some("Use unique node ids".to_string()),
                );
            }
            if !is_valid_canonical_name(&node.id) {
                emit_rule(
                    &mut diagnostics,
                    "E1007",
                    format!("illegal node id: {}", node.id),
                    format!("/nodes/{}", node.id),
                    Some("Use [a-zA-Z0-9_-] only".to_string()),
                );
            }
            for tag in &node.tags {
                if !is_valid_canonical_name(tag) {
                    emit_rule(
                        &mut diagnostics,
                        "E1026",
                        format!("illegal node tag: {}", tag),
                        format!("/nodes/{}/tags", node.id),
                        Some("Use [a-zA-Z0-9_-] only".to_string()),
                    );
                }
            }
            if node.kind == crate::NodeKind::Shell && node.effects.is_empty() {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("missing effects for shell node: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Declare effects for shell nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Shell && !node.effects.contains(&Effect::Filesystem) {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("shell node missing filesystem effect: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Include filesystem effect for shell nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Python && node.effects.is_empty() {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("missing effects for python node: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Declare effects for python nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Python && !node.effects.contains(&Effect::Filesystem) {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("python node missing filesystem effect: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Include filesystem effect for python nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Python && !matches!(node.params, ParamValue::Object(_))
            {
                emit_rule(
                    &mut diagnostics,
                    "E1039",
                    format!("python node params must be an object: {}", node.id),
                    format!("/nodes/{}/params", node.id),
                    Some("Declare python module and function inside params".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Python {
                match node_param_literal_string(node, "module") {
                    Some(module) if !module.trim().is_empty() => {}
                    _ => emit_rule(
                        &mut diagnostics,
                        "E1040",
                        format!("python node missing module: {}", node.id),
                        format!("/nodes/{}/params/module", node.id),
                        Some("Provide a non-empty module string for python nodes".to_string()),
                    ),
                }
                match node_param_literal_string(node, "function") {
                    Some(function) if !function.trim().is_empty() => {}
                    _ => emit_rule(
                        &mut diagnostics,
                        "E1041",
                        format!("python node missing function: {}", node.id),
                        format!("/nodes/{}/params/function", node.id),
                        Some("Provide a non-empty function string for python nodes".to_string()),
                    ),
                }
            }
            if node.kind == crate::NodeKind::Http && node.effects.is_empty() {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("missing effects for http node: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Declare effects for http nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Http && !node.effects.contains(&Effect::Filesystem) {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("http node missing filesystem effect: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Include filesystem effect for http nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Http && !node.effects.contains(&Effect::Network) {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("http node missing network effect: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Include network effect for http nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Http && !matches!(node.params, ParamValue::Object(_)) {
                emit_rule(
                    &mut diagnostics,
                    "E1042",
                    format!("http node params must be an object: {}", node.id),
                    format!("/nodes/{}/params", node.id),
                    Some("Declare method, url, headers, and body inside params".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Http {
                match node_param_literal_string(node, "method") {
                    Some(method) if !method.trim().is_empty() => {}
                    _ => emit_rule(
                        &mut diagnostics,
                        "E1043",
                        format!("http node missing method: {}", node.id),
                        format!("/nodes/{}/params/method", node.id),
                        Some("Provide a non-empty HTTP method string".to_string()),
                    ),
                }
                match node_param_literal_string(node, "url") {
                    Some(url) if url.starts_with("http://") || url.starts_with("https://") => {}
                    _ => emit_rule(
                        &mut diagnostics,
                        "E1044",
                        format!("http node missing or invalid url: {}", node.id),
                        format!("/nodes/{}/params/url", node.id),
                        Some("Provide an absolute http:// or https:// URL".to_string()),
                    ),
                }
                if let Some(headers) = node_param_object_field(node, "headers") {
                    if headers.values().any(|value| !param_value_is_literal_string(value)) {
                        emit_rule(
                            &mut diagnostics,
                            "E1045",
                            format!("http node headers must be string literals: {}", node.id),
                            format!("/nodes/{}/params/headers", node.id),
                            Some(
                                "Provide request headers as an object of string values".to_string(),
                            ),
                        );
                    }
                } else if node_param_object(node)
                    .is_some_and(|params| params.contains_key("headers"))
                {
                    emit_rule(
                        &mut diagnostics,
                        "E1045",
                        format!("http node headers must be an object: {}", node.id),
                        format!("/nodes/{}/params/headers", node.id),
                        Some("Provide request headers as an object of string values".to_string()),
                    );
                }
                if node.outputs.len() != 1 {
                    emit_rule(
                        &mut diagnostics,
                        "E1046",
                        format!("http node requires exactly one declared output: {}", node.id),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Declare exactly one output to capture the HTTP response".to_string()),
                    );
                }
            }
            if node.kind == crate::NodeKind::FileTransform && node.effects.is_empty() {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("missing effects for file_transform node: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Declare effects for file_transform nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::FileTransform
                && !node.effects.contains(&Effect::Filesystem)
            {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("file_transform node missing filesystem effect: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Include filesystem effect for file_transform nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::FileTransform
                && !matches!(node.params, ParamValue::Object(_))
            {
                emit_rule(
                    &mut diagnostics,
                    "E1047",
                    format!("file_transform node params must be an object: {}", node.id),
                    format!("/nodes/{}/params", node.id),
                    Some(
                        "Declare operation-specific fields inside an object for file_transform nodes"
                            .to_string(),
                    ),
                );
            }
            if node.kind == crate::NodeKind::FileTransform {
                let operation = match node_param_literal_string(node, "operation") {
                    Some(
                        "copy" | "concatenate" | "split" | "gzip_compress" | "gzip_decompress"
                        | "checksum",
                    ) => node_param_literal_string(node, "operation"),
                    _ => {
                        emit_rule(
                            &mut diagnostics,
                            "E1048",
                            format!(
                                "file_transform node operation must be one of copy, concatenate, split, gzip_compress, gzip_decompress, checksum: {}",
                                node.id
                            ),
                            format!("/nodes/{}/params/operation", node.id),
                            Some("Choose a supported built-in file_transform operation".to_string()),
                        );
                        None
                    }
                };

                let requires_single_source = matches!(
                    operation,
                    Some("copy" | "split" | "gzip_compress" | "gzip_decompress" | "checksum")
                );
                if requires_single_source {
                    match node_param_literal_string(node, "source") {
                        Some(path) if is_normalized_relative_path(path) => {}
                        _ => emit_rule(
                            &mut diagnostics,
                            "E1049",
                            format!(
                                "file_transform node source must be a normalized relative input path: {}",
                                node.id
                            ),
                            format!("/nodes/{}/params/source", node.id),
                            Some(
                                "Reference an input artifact relative to the node inputs directory"
                                    .to_string(),
                            ),
                        ),
                    }
                }

                if matches!(operation, Some("concatenate")) {
                    match node_param_array_field(node, "sources") {
                        Some(paths)
                            if !paths.is_empty()
                                && paths.iter().all(|value| {
                                    matches!(
                                        value,
                                        ParamValue::Literal(serde_json::Value::String(path))
                                            if is_normalized_relative_path(path)
                                    )
                                }) => {}
                        _ => emit_rule(
                            &mut diagnostics,
                            "E1050",
                            format!(
                                "file_transform node sources must be a non-empty array of normalized relative input paths: {}",
                                node.id
                            ),
                            format!("/nodes/{}/params/sources", node.id),
                            Some(
                                "Provide concatenate sources as an ordered array of input-relative paths"
                                    .to_string(),
                            ),
                        ),
                    }
                }

                if matches!(operation, Some("split")) {
                    match node_param_object(node)
                        .and_then(|params| params.get("chunk_bytes"))
                        .and_then(param_value_literal_u64)
                    {
                        Some(chunk_bytes) if chunk_bytes > 0 => {}
                        _ => emit_rule(
                            &mut diagnostics,
                            "E1051",
                            format!(
                                "file_transform split node requires chunk_bytes > 0: {}",
                                node.id
                            ),
                            format!("/nodes/{}/params/chunk_bytes", node.id),
                            Some(
                                "Provide a positive chunk_bytes integer for split operations"
                                    .to_string(),
                            ),
                        ),
                    }
                }

                if matches!(operation, Some("checksum"))
                    && node_param_object(node)
                        .is_some_and(|params| params.contains_key("checksum_algorithm"))
                {
                    match node_param_literal_string(node, "checksum_algorithm") {
                        Some("sha256") => {}
                        _ => emit_rule(
                            &mut diagnostics,
                            "E1052",
                            format!(
                                "file_transform checksum_algorithm must be sha256 when provided: {}",
                                node.id
                            ),
                            format!("/nodes/{}/params/checksum_algorithm", node.id),
                            Some("Use checksum_algorithm: \"sha256\"".to_string()),
                        ),
                    }
                }

                if matches!(operation, Some("gzip_compress"))
                    && node_param_object(node)
                        .is_some_and(|params| params.contains_key("compression_level"))
                {
                    match node_param_object(node)
                        .and_then(|params| params.get("compression_level"))
                        .and_then(param_value_literal_u64)
                    {
                        Some(level) if level <= 9 => {}
                        _ => emit_rule(
                            &mut diagnostics,
                            "E1053",
                            format!(
                                "file_transform compression_level must be an integer between 0 and 9: {}",
                                node.id
                            ),
                            format!("/nodes/{}/params/compression_level", node.id),
                            Some("Use gzip compression levels from 0 through 9".to_string()),
                        ),
                    }
                }

                if node.outputs.iter().any(|output| output.expects_directory()) {
                    emit_rule(
                        &mut diagnostics,
                        "E1054",
                        format!(
                            "file_transform node outputs must be file outputs, not directories: {}",
                            node.id
                        ),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Declare file outputs for file_transform operations".to_string()),
                    );
                }

                match operation {
                    Some("split") if node.outputs.is_empty() => emit_rule(
                        &mut diagnostics,
                        "E1055",
                        format!(
                            "file_transform split node requires one or more declared outputs: {}",
                            node.id
                        ),
                        format!("/nodes/{}/outputs", node.id),
                        Some(
                            "Declare outputs for each split chunk you expect to materialize"
                                .to_string(),
                        ),
                    ),
                    Some(
                        "copy" | "concatenate" | "gzip_compress" | "gzip_decompress" | "checksum",
                    ) if node.outputs.len() != 1 => {
                        emit_rule(
                            &mut diagnostics,
                            "E1055",
                            format!(
                                "file_transform node requires exactly one declared output for operation {}: {}",
                                operation.unwrap_or("unknown"),
                                node.id
                            ),
                            format!("/nodes/{}/outputs", node.id),
                            Some(
                                "Declare exactly one output for copy, concatenate, gzip, and checksum operations"
                                    .to_string(),
                            ),
                        );
                    }
                    _ => {}
                }
            }
            if node.kind == crate::NodeKind::Container && node.container.is_none() {
                emit_rule(
                    &mut diagnostics,
                    "E1023",
                    format!("missing container spec for node: {}", node.id),
                    format!("/nodes/{}/container", node.id),
                    Some("Provide container spec for container nodes".to_string()),
                );
            }
            if node.kind == crate::NodeKind::Container
                && !node.effects.contains(&Effect::Filesystem)
            {
                emit_rule(
                    &mut diagnostics,
                    "E1009",
                    format!("container node missing filesystem effect: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Include filesystem effect for container nodes".to_string()),
                );
            }
            if node.retry.max_attempts > 0
                && (node.effects.contains(&Effect::Clock)
                    || node.effects.contains(&Effect::Network))
            {
                let has_seed = self.inputs.contains_key("random_seed");
                if !has_seed && !self.nondeterminism_allowed {
                    emit_rule(
                        &mut diagnostics,
                        "E1011",
                        format!("retry not allowed for nondeterministic node: {}", node.id),
                        format!("/nodes/{}/retry", node.id),
                        Some(
                            "Provide inputs.random_seed or set nondeterminism_allowed=true"
                                .to_string(),
                        ),
                    );
                }
            }
            if node.cache.enabled {
                if let Some(reason) = &node.cache.reason {
                    if !reason.trim().is_empty() {
                        emit_rule(
                            &mut diagnostics,
                            "E1032",
                            format!(
                                "cache reason only applies when cache is disabled: {}",
                                node.id
                            ),
                            format!("/nodes/{}/cache/reason", node.id),
                            Some("Remove cache.reason or set cache.enabled to false".to_string()),
                        );
                    }
                }
            } else if node.cache.reason.as_deref().is_none_or(|reason| reason.trim().is_empty()) {
                emit_rule(
                    &mut diagnostics,
                    "E1032",
                    format!("cache-disabled node requires a reason: {}", node.id),
                    format!("/nodes/{}/cache", node.id),
                    Some("Provide cache.reason when cache.enabled is false".to_string()),
                );
            }
            let container_env_allowlist =
                node.container.as_ref().map(|spec| spec.env_allowlist.as_slice()).unwrap_or(&[]);
            let has_declared_env_bindings =
                !node.env_allowlist.is_empty() || !container_env_allowlist.is_empty();
            if !node.env_allowlist.is_empty() && !node.effects.contains(&Effect::Env) {
                emit_rule(
                    &mut diagnostics,
                    "E1010",
                    format!("env_allowlist without env effect: {}", node.id),
                    format!("/nodes/{}/env_allowlist", node.id),
                    Some("Add env effect when using env_allowlist".to_string()),
                );
            }
            for entry in &node.env_allowlist {
                if !valid_env_allowlist_pattern(entry) {
                    emit_rule(
                        &mut diagnostics,
                        "E1027",
                        format!("invalid env allowlist entry: {}", entry),
                        format!("/nodes/{}/env_allowlist", node.id),
                        Some(
                            "Use letters, digits, underscores, and optional suffix '*'".to_string(),
                        ),
                    );
                }
            }
            if node.effects.contains(&Effect::Env) && !has_declared_env_bindings {
                emit_rule(
                    &mut diagnostics,
                    "E1035",
                    format!("env effect requires declared env_allowlist bindings: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some(
                        "Declare allowed environment variables in env_allowlist or container.env_allowlist"
                            .to_string(),
                    ),
                );
            }
            if node.kind == crate::NodeKind::Container {
                if let Some(spec) = &node.container {
                    if !spec.env_allowlist.is_empty() && !node.effects.contains(&Effect::Env) {
                        emit_rule(
                            &mut diagnostics,
                            "E1010",
                            format!("container env_allowlist without env effect: {}", node.id),
                            format!("/nodes/{}/container/env_allowlist", node.id),
                            Some("Add env effect when using env_allowlist".to_string()),
                        );
                    }
                    for entry in &spec.env_allowlist {
                        if !valid_env_allowlist_pattern(entry) {
                            emit_rule(
                                &mut diagnostics,
                                "E1027",
                                format!("invalid container env allowlist entry: {}", entry),
                                format!("/nodes/{}/container/env_allowlist", node.id),
                                Some(
                                    "Use letters, digits, underscores, and optional suffix '*'"
                                        .to_string(),
                                ),
                            );
                        }
                    }
                    if spec.engine != "docker" && spec.engine != "podman" {
                        emit_rule(
                            &mut diagnostics,
                            "E1024",
                            format!("invalid container engine: {}", spec.engine),
                            format!("/nodes/{}/container/engine", node.id),
                            Some("Use engine \"docker\" or \"podman\"".to_string()),
                        );
                    }
                    if spec.argv.is_empty() {
                        emit_rule(
                            &mut diagnostics,
                            "E1024",
                            format!("container argv must not be empty: {}", node.id),
                            format!("/nodes/{}/container/argv", node.id),
                            Some("Provide argv for container nodes".to_string()),
                        );
                    }
                    if let Some(workdir) = spec.workdir.as_deref() {
                        if let Err(message) = validate_container_workdir_value(workdir) {
                            emit_rule(
                                &mut diagnostics,
                                "E1025",
                                message,
                                format!("/nodes/{}/container/workdir", node.id),
                                Some(
                                    "Use an absolute path or a normalized relative/path-variable workdir"
                                        .to_string(),
                                ),
                            );
                        }
                    }
                }
            }
            match (&node.semantic_kind, &node.branch) {
                (SemanticNodeKind::Branch, None) => {
                    emit_rule(
                        &mut diagnostics,
                        "E1028",
                        format!("branch node missing branch contract: {}", node.id),
                        format!("/nodes/{}/branch", node.id),
                        Some(
                            "Provide decisions and a decision_output for branch nodes".to_string(),
                        ),
                    );
                }
                (SemanticNodeKind::Branch, Some(branch)) => {
                    let mut decisions = BTreeSet::new();
                    let mut duplicate_decisions = BTreeSet::new();
                    for decision in &branch.decisions {
                        if !is_valid_canonical_name(decision) {
                            emit_rule(
                                &mut diagnostics,
                                "E1028",
                                format!("illegal branch decision: {}", decision),
                                format!("/nodes/{}/branch/decisions", node.id),
                                Some("Use canonical names for branch decisions".to_string()),
                            );
                        }
                        if !decisions.insert(decision.clone()) {
                            duplicate_decisions.insert(decision.clone());
                        }
                    }
                    if branch.decisions.is_empty() {
                        emit_rule(
                            &mut diagnostics,
                            "E1028",
                            format!("branch node must declare at least one decision: {}", node.id),
                            format!("/nodes/{}/branch/decisions", node.id),
                            Some("Declare one or more named branch decisions".to_string()),
                        );
                    }
                    for decision in duplicate_decisions {
                        emit_rule(
                            &mut diagnostics,
                            "E1028",
                            format!("duplicate branch decision: {}", decision),
                            format!("/nodes/{}/branch/decisions", node.id),
                            Some("Make each branch decision unique".to_string()),
                        );
                    }
                    if !node.outputs.iter().any(|output| output.name == branch.decision_output) {
                        emit_rule(
                            &mut diagnostics,
                            "E1028",
                            format!(
                                "branch decision output '{}' is not declared on node {}",
                                branch.decision_output, node.id
                            ),
                            format!("/nodes/{}/branch/decision_output", node.id),
                            Some("decision_output must match a declared node output".to_string()),
                        );
                    }
                    if let Some(default_decision) = &branch.default_decision {
                        if !decisions.contains(default_decision) {
                            emit_rule(
                                &mut diagnostics,
                                "E1028",
                                format!(
                                    "default branch decision '{}' is not declared on node {}",
                                    default_decision, node.id
                                ),
                                format!("/nodes/{}/branch/default_decision", node.id),
                                Some(
                                    "default_decision must be one of branch.decisions".to_string(),
                                ),
                            );
                        }
                    }
                    branch_output_by_node.insert(node.id.as_str(), branch.decision_output.as_str());
                    branch_decisions_by_node.insert(node.id.as_str(), decisions);
                }
                (_, Some(_)) => {
                    emit_rule(
                        &mut diagnostics,
                        "E1028",
                        format!(
                            "branch contract is only allowed on semantic_kind=branch nodes: {}",
                            node.id
                        ),
                        format!("/nodes/{}/branch", node.id),
                        Some("Set semantic_kind=branch or remove the branch contract".to_string()),
                    );
                }
                _ => {}
            }
            match (&node.semantic_kind, &node.dynamic) {
                (SemanticNodeKind::Dynamic, None) => {
                    emit_rule(
                        &mut diagnostics,
                        "E1063",
                        format!("dynamic node missing dynamic contract: {}", node.id),
                        format!("/nodes/{}/dynamic", node.id),
                        Some(
                            "Declare dynamic.expansion_output so the runtime can load the generated graph fragment"
                                .to_string(),
                        ),
                    );
                }
                (SemanticNodeKind::Dynamic, Some(dynamic)) => {
                    dynamic_nodes.insert(node.id.clone());
                    if !node.outputs.iter().any(|output| output.name == dynamic.expansion_output) {
                        emit_rule(
                            &mut diagnostics,
                            "E1064",
                            format!(
                                "dynamic expansion output '{}' is not declared on node {}",
                                dynamic.expansion_output, node.id
                            ),
                            format!("/nodes/{}/dynamic/expansion_output", node.id),
                            Some(
                                "Declare the expansion output as a normal node output so the controller run persists it"
                                    .to_string(),
                            ),
                        );
                    }
                    if node.outputs.iter().any(|output| {
                        output.name == dynamic.expansion_output && output.expects_directory()
                    }) {
                        emit_rule(
                            &mut diagnostics,
                            "E1065",
                            format!(
                                "dynamic expansion output '{}' on node {} must be a file or value output",
                                dynamic.expansion_output, node.id
                            ),
                            format!("/nodes/{}/outputs", node.id),
                            Some(
                                "Write one expansion document file that declares generated nodes and edges"
                                    .to_string(),
                            ),
                        );
                    }
                    if !node.inputs.is_empty() {
                        emit_rule(
                            &mut diagnostics,
                            "E1067",
                            format!(
                                "dynamic controller nodes must not declare runtime inputs: {}",
                                node.id
                            ),
                            format!("/nodes/{}/inputs", node.id),
                            Some(
                                "Use graph inputs inside the controller command; dynamic expansion runs before normal node-to-node execution"
                                    .to_string(),
                            ),
                        );
                    }
                    if node.branch.is_some() {
                        emit_rule(
                            &mut diagnostics,
                            "E1068",
                            format!(
                                "dynamic controller nodes cannot also declare branch contracts: {}",
                                node.id
                            ),
                            format!("/nodes/{}/branch", node.id),
                            Some(
                                "Keep dynamic expansion and branch routing as separate node responsibilities"
                                    .to_string(),
                            ),
                        );
                    }
                }
                (_, Some(_)) => {
                    emit_rule(
                        &mut diagnostics,
                        "E1063",
                        format!(
                            "dynamic contract is only allowed on semantic_kind=dynamic nodes: {}",
                            node.id
                        ),
                        format!("/nodes/{}/dynamic", node.id),
                        Some(
                            "Set semantic_kind=dynamic or remove the dynamic contract".to_string(),
                        ),
                    );
                }
                _ => {}
            }
            if node.semantic_kind == SemanticNodeKind::Map {
                if node.outputs.is_empty() {
                    emit_rule(
                        &mut diagnostics,
                        "E1056",
                        format!("map node requires one or more declared outputs: {}", node.id),
                        format!("/nodes/{}/outputs", node.id),
                        Some(
                            "Declare directory outputs that will collect per-item map results"
                                .to_string(),
                        ),
                    );
                }
                if node.outputs.iter().any(|output| output.expects_file()) {
                    emit_rule(
                        &mut diagnostics,
                        "E1057",
                        format!("map node outputs must be directory outputs: {}", node.id),
                        format!("/nodes/{}/outputs", node.id),
                        Some(
                            "Use output kind directory so each mapped item has an isolated artifact root"
                                .to_string(),
                        ),
                    );
                }
                if node.inputs.is_empty() {
                    emit_rule(
                        &mut diagnostics,
                        "E1058",
                        format!("map node requires at least one declared input: {}", node.id),
                        format!("/nodes/{}/inputs", node.id),
                        Some(
                            "Bind a JSON array input that the runtime can expand deterministically"
                                .to_string(),
                        ),
                    );
                } else if node.inputs.len() > 1 {
                    match node_param_object_field(node, "map")
                        .and_then(|map| map.get("input"))
                        .and_then(param_value_literal_string)
                    {
                        Some(input) if node.inputs.iter().any(|candidate| candidate == input) => {}
                        Some(input) => emit_rule(
                            &mut diagnostics,
                            "E1058",
                            format!(
                                "map.input '{}' is not a declared input on node {}",
                                input, node.id
                            ),
                            format!("/nodes/{}/params/map/input", node.id),
                            Some(
                                "Choose one of the declared inputs as the array source for semantic map expansion"
                                    .to_string(),
                            ),
                        ),
                        None => emit_rule(
                            &mut diagnostics,
                            "E1058",
                            format!(
                                "map node with multiple inputs must declare params.map.input: {}",
                                node.id
                            ),
                            format!("/nodes/{}/params/map/input", node.id),
                            Some(
                                "Set params.map.input to the input port that carries the JSON array"
                                    .to_string(),
                            ),
                        ),
                    }
                }
            }
            if node.semantic_kind == SemanticNodeKind::Reduce {
                if node.outputs.len() != 1 {
                    emit_rule(
                        &mut diagnostics,
                        "E1059",
                        format!("reduce node must declare exactly one output: {}", node.id),
                        format!("/nodes/{}/outputs", node.id),
                        Some(
                            "Declare one reducer output artifact so fan-in has a single result contract"
                                .to_string(),
                        ),
                    );
                }
                if node.trigger_rule != TriggerRule::AllSuccess {
                    emit_rule(
                        &mut diagnostics,
                        "E1062",
                        format!("reduce node trigger_rule must remain all_success: {}", node.id),
                        format!("/nodes/{}/trigger_rule", node.id),
                        Some(
                            "Use params.reduce.mode to choose all_success or partial reducer behavior"
                                .to_string(),
                        ),
                    );
                }
                if let Some(reduce) = node_param_object_field(node, "reduce") {
                    if reduce
                        .get("allow_empty_collection")
                        .and_then(param_value_literal_bool)
                        .is_some()
                    {
                        emit_rule(
                            &mut diagnostics,
                            "E1061",
                            format!(
                                "reduce.allow_empty_collection is not supported on node {}",
                                node.id
                            ),
                            format!("/nodes/{}/params/reduce/allow_empty_collection", node.id),
                            Some("Use params.reduce.empty with forbid, allow, or skip".to_string()),
                        );
                    }
                    if let Some(mode) = reduce.get("mode").and_then(param_value_literal_string) {
                        if !matches!(mode, "all_success" | "partial") {
                            emit_rule(
                                &mut diagnostics,
                                "E1060",
                                format!(
                                    "reduce.mode '{}' is not supported on node {}",
                                    mode, node.id
                                ),
                                format!("/nodes/{}/params/reduce/mode", node.id),
                                Some(
                                    "Choose reduce.mode=all_success or reduce.mode=partial"
                                        .to_string(),
                                ),
                            );
                        }
                    }
                    if let Some(empty_policy) =
                        reduce.get("empty").and_then(param_value_literal_string)
                    {
                        if !matches!(empty_policy, "forbid" | "allow" | "skip") {
                            emit_rule(
                                &mut diagnostics,
                                "E1061",
                                format!(
                                    "reduce.empty '{}' is not supported on node {}",
                                    empty_policy, node.id
                                ),
                                format!("/nodes/{}/params/reduce/empty", node.id),
                                Some("Choose reduce.empty=forbid, allow, or skip".to_string()),
                            );
                        }
                    }
                }
            }
            node_map.insert(node.id.as_str(), node);
        }

        let mut edge_pairs = BTreeSet::new();
        let mut target_bindings = BTreeSet::new();
        let mut bound_inputs = BTreeSet::<(String, String)>::new();
        let mut conditional_edge_counts = HashMap::<(String, String), usize>::new();
        let mut conditional_incoming_targets = BTreeSet::new();
        for edge in &self.edges {
            if dynamic_nodes.contains(&edge.from.node_id)
                || dynamic_nodes.contains(&edge.to.node_id)
            {
                let controller_node_id = if dynamic_nodes.contains(&edge.from.node_id) {
                    &edge.from.node_id
                } else {
                    &edge.to.node_id
                };
                emit_rule(
                    &mut diagnostics,
                    "E1066",
                    format!(
                        "dynamic controller node {} must not have declared graph edges",
                        controller_node_id
                    ),
                    format!("/edges/{}->{}", edge.from.node_id, edge.to.node_id),
                    Some(
                        "Emit connectivity in the generated expansion document instead of wiring the controller node into the static graph"
                            .to_string(),
                    ),
                );
            }
            let from_node = node_map.get(edge.from.node_id.as_str());
            let to_node = node_map.get(edge.to.node_id.as_str());
            if from_node.is_none() {
                emit_rule(
                    &mut diagnostics,
                    "E1002",
                    format!("dangling node reference: {}", edge.from.node_id),
                    format!("/edges/from/{}", edge.from.node_id),
                    None,
                );
                continue;
            }
            if to_node.is_none() {
                emit_rule(
                    &mut diagnostics,
                    "E1002",
                    format!("dangling node reference: {}", edge.to.node_id),
                    format!("/edges/to/{}", edge.to.node_id),
                    None,
                );
                continue;
            }
            let from_node = from_node.expect("checked above");
            let to_node = to_node.expect("checked above");

            if !from_node.outputs.iter().any(|output| output.name == edge.from.port) {
                emit_rule(
                    &mut diagnostics,
                    "E1003",
                    format!("dangling port reference: {}.{}", edge.from.node_id, edge.from.port),
                    format!("/edges/from/{}/{}", edge.from.node_id, edge.from.port),
                    None,
                );
            }
            if !to_node.inputs.iter().any(|input| input == &edge.to.port) {
                emit_rule(
                    &mut diagnostics,
                    "E1003",
                    format!("dangling port reference: {}.{}", edge.to.node_id, edge.to.port),
                    format!("/edges/to/{}/{}", edge.to.node_id, edge.to.port),
                    None,
                );
            }
            if from_node.outputs.iter().any(|output| output.name == edge.from.port)
                && to_node.inputs.iter().any(|input| input == &edge.to.port)
            {
                bound_inputs.insert((edge.to.node_id.clone(), edge.to.port.clone()));
            }

            let pair_key = format!(
                "{}:{}->{}:{}",
                edge.from.node_id, edge.from.port, edge.to.node_id, edge.to.port
            );
            if !edge_pairs.insert(pair_key) {
                emit_rule(
                    &mut diagnostics,
                    "E1008",
                    "output collision".to_string(),
                    "/edges".to_string(),
                    Some("Avoid duplicate edge targets".to_string()),
                );
            }

            let target_key = format!("{}:{}", edge.to.node_id, edge.to.port);
            if !target_bindings.insert(target_key) {
                emit_rule(
                    &mut diagnostics,
                    "E1008",
                    format!(
                        "ambiguous dependency binding for input {}.{}",
                        edge.to.node_id, edge.to.port
                    ),
                    format!("/edges/to/{}/{}", edge.to.node_id, edge.to.port),
                    Some("Bind each input port from exactly one source output".to_string()),
                );
            }

            if edge.kind == EdgeKind::Conditional {
                conditional_incoming_targets.insert(edge.to.node_id.clone());
                match branch_decisions_by_node.get(edge.from.node_id.as_str()) {
                    None => emit_rule(
                        &mut diagnostics,
                        "E1029",
                        format!(
                            "conditional edge must originate from a branch node: {}.{}",
                            edge.from.node_id, edge.from.port
                        ),
                        format!("/edges/from/{}/{}", edge.from.node_id, edge.from.port),
                        Some("Use semantic_kind=branch on the source node".to_string()),
                    ),
                    Some(decisions) => {
                        let expected_output =
                            branch_output_by_node.get(edge.from.node_id.as_str()).copied();
                        if expected_output != Some(edge.from.port.as_str()) {
                            emit_rule(
                                &mut diagnostics,
                                "E1029",
                                format!(
                                    "conditional edge must read branch decision output {}.{}",
                                    edge.from.node_id,
                                    expected_output.unwrap_or("<missing>")
                                ),
                                format!("/edges/from/{}/{}", edge.from.node_id, edge.from.port),
                                Some(
                                    "Point conditional edges at the declared decision_output"
                                        .to_string(),
                                ),
                            );
                        }
                        match &edge.decision {
                            None => emit_rule(
                                &mut diagnostics,
                                "E1029",
                                format!(
                                    "conditional edge missing decision label: {} -> {}",
                                    edge.from.node_id, edge.to.node_id
                                ),
                                "/edges".to_string(),
                                Some("Set edge.decision for conditional edges".to_string()),
                            ),
                            Some(decision) if !decisions.contains(decision) => emit_rule(
                                &mut diagnostics,
                                "E1029",
                                format!(
                                    "unknown branch decision '{}' on edge {} -> {}",
                                    decision, edge.from.node_id, edge.to.node_id
                                ),
                                "/edges".to_string(),
                                Some("Use one of the source branch node decisions".to_string()),
                            ),
                            Some(decision) => {
                                *conditional_edge_counts
                                    .entry((edge.from.node_id.clone(), decision.clone()))
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                }
            } else if branch_output_by_node.get(edge.from.node_id.as_str()).copied()
                == Some(edge.from.port.as_str())
            {
                emit_rule(
                    &mut diagnostics,
                    "E1030",
                    format!(
                        "branch decision output {}.{} must only drive conditional edges",
                        edge.from.node_id, edge.from.port
                    ),
                    format!("/edges/from/{}/{}", edge.from.node_id, edge.from.port),
                    Some("Use edge.kind=conditional with a valid edge.decision".to_string()),
                );
            }
        }

        for node in &self.nodes {
            for input in &node.inputs {
                if !bound_inputs.contains(&(node.id.clone(), input.clone())) {
                    emit_rule(
                        &mut diagnostics,
                        "E1005",
                        format!("missing required input binding: {}.{}", node.id, input),
                        format!("/nodes/{}/inputs/{}", node.id, input),
                        Some("Connect an upstream edge for every declared node input".to_string()),
                    );
                }
            }
            if let Some(decisions) = branch_decisions_by_node.get(node.id.as_str()) {
                for decision in decisions {
                    if conditional_edge_counts
                        .get(&(node.id.clone(), decision.clone()))
                        .copied()
                        .unwrap_or_default()
                        == 0
                    {
                        emit_rule(
                            &mut diagnostics,
                            "E1028",
                            format!(
                                "branch decision '{}' on node {} has no conditional edge",
                                decision, node.id
                            ),
                            format!("/nodes/{}/branch/decisions", node.id),
                            Some(
                                "Add a conditional edge for every declared branch decision"
                                    .to_string(),
                            ),
                        );
                    }
                }
            }
            if conditional_incoming_targets.contains(&node.id)
                && !trigger_rule_supports_conditional_incoming(&node.trigger_rule)
            {
                emit_rule(
                    &mut diagnostics,
                    "E1030",
                    format!(
                        "trigger_rule {:?} is incompatible with conditional incoming edges on node {}",
                        node.trigger_rule, node.id
                    ),
                    format!("/nodes/{}/trigger_rule", node.id),
                    Some("Use any_success or all_done for nodes fed by conditional edges".to_string()),
                );
            }
        }

        for node in &self.nodes {
            let mut seen_outputs = BTreeSet::new();
            for output in &node.outputs {
                if !is_valid_output_path(&output.path) {
                    emit_rule(
                        &mut diagnostics,
                        "E1025",
                        format!("invalid output path: {}", output.path),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Use relative paths without '..'".to_string()),
                    );
                }
                if !seen_outputs.insert(output.name.as_str()) {
                    emit_rule(
                        &mut diagnostics,
                        "E1008",
                        format!("duplicate output name: {}", output.name),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Make output names unique per node".to_string()),
                    );
                }
            }
        }

        let mut output_paths = BTreeSet::new();
        for node in &self.nodes {
            for output in &node.outputs {
                if !output_paths.insert(output.path.as_str()) {
                    emit_rule(
                        &mut diagnostics,
                        "E1008",
                        format!("output collision: {}", output.path),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Avoid duplicate output paths across nodes".to_string()),
                    );
                }
            }
        }

        if self.has_cycle() {
            emit_rule(
                &mut diagnostics,
                "E1004",
                "cycle detected".to_string(),
                "/edges".to_string(),
                None,
            );
        }

        diagnostics.extend(self.validate_param_refs());
        diagnostics.extend(self.unreachable_warnings());
        diagnostics.extend(self.orphan_warnings());
        diagnostics.extend(self.validate_graph_meta_names());
        diagnostics.sort_by(|left, right| {
            severity_rank(&left.severity)
                .cmp(&severity_rank(&right.severity))
                .then_with(|| left.code.cmp(&right.code))
                .then_with(|| left.path.cmp(&right.path))
                .then_with(|| left.message.cmp(&right.message))
                .then_with(|| left.hint.cmp(&right.hint))
        });

        diagnostics
    }

    pub fn validate_strict(&self) -> Result<Vec<ValidationDiagnostic>, GraphError> {
        let diagnostics = self.validate_with_warnings();
        if diagnostics.iter().any(|diag| diag.severity == Severity::Error) {
            return Err(GraphError::ValidationFailed);
        }
        Ok(diagnostics)
    }

    fn unreachable_warnings(&self) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            adjacency.entry(edge.from.node_id.as_str()).or_default().push(edge.to.node_id.as_str());
        }

        let mut visited = BTreeSet::new();
        let mut stack: Vec<&str> = self
            .nodes
            .iter()
            .filter_map(|node| if node.inputs.is_empty() { Some(node.id.as_str()) } else { None })
            .collect();

        while let Some(node_id) = stack.pop() {
            if !visited.insert(node_id) {
                continue;
            }
            if let Some(next_nodes) = adjacency.get(node_id) {
                for next in next_nodes {
                    stack.push(next);
                }
            }
        }

        for node in &self.nodes {
            if !visited.contains(node.id.as_str()) {
                emit_rule(
                    &mut diagnostics,
                    "W2001",
                    format!("unreachable node: {}", node.id),
                    format!("/nodes/{}", node.id),
                    None,
                );
            }
        }

        diagnostics
    }

    fn orphan_warnings(&self) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        let mut involved = BTreeSet::new();
        for edge in &self.edges {
            involved.insert(edge.from.node_id.as_str());
            involved.insert(edge.to.node_id.as_str());
        }
        for node in &self.nodes {
            if !involved.contains(node.id.as_str()) {
                emit_rule(
                    &mut diagnostics,
                    "W2002",
                    format!("orphan node: {}", node.id),
                    format!("/nodes/{}", node.id),
                    None,
                );
            }
        }
        diagnostics
    }

    fn validate_param_refs(&self) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        let topo_order = self.topo_order().ok();
        let mut indices = HashMap::new();
        if let Some(order) = topo_order.as_ref() {
            for (index, node_id) in order.iter().enumerate() {
                indices.insert(node_id.as_str(), index);
            }
        }
        for node in &self.nodes {
            validate_param_value(
                &node.params,
                self,
                node,
                &indices,
                &mut diagnostics,
                &format!("/nodes/{}/params", node.id),
            );
        }
        diagnostics
    }

    fn validate_graph_meta_names(&self) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();
        if let Some(meta) = &self.meta {
            if !is_valid_canonical_name(&meta.name) {
                emit_rule(
                    &mut diagnostics,
                    "E1027",
                    format!("illegal graph name: {}", meta.name),
                    "/meta/name".to_string(),
                    Some("Use [a-zA-Z0-9_-] only".to_string()),
                );
            }
            for tag in &meta.tags {
                if !is_valid_canonical_name(tag) {
                    emit_rule(
                        &mut diagnostics,
                        "E1026",
                        format!("illegal graph tag: {}", tag),
                        "/meta/tags".to_string(),
                        Some("Use [a-zA-Z0-9_-] only".to_string()),
                    );
                }
            }
        }
        diagnostics
    }
}

fn validate_graph_inputs(graph: &Graph, diagnostics: &mut Vec<ValidationDiagnostic>) {
    for (input_name, spec) in &graph.inputs {
        validate_graph_input_spec(spec, &format!("/inputs/{input_name}"), diagnostics);
    }
}

fn validate_graph_input_spec(
    spec: &GraphInputSpec,
    path: &str,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    match &spec.kind {
        GraphInputKind::Enum { values } => {
            if values.is_empty() {
                emit_rule(
                    diagnostics,
                    "E1034",
                    "enum input must declare at least one allowed value".to_string(),
                    format!("{path}/values"),
                    Some("Provide one or more enum values".to_string()),
                );
            }
            let mut seen = BTreeSet::new();
            for value in values {
                if !seen.insert(value) {
                    emit_rule(
                        diagnostics,
                        "E1034",
                        format!("duplicate enum value: {value}"),
                        format!("{path}/values"),
                        Some("Make enum values unique".to_string()),
                    );
                }
            }
        }
        GraphInputKind::Array { items } => {
            if let Some(item_kind) = items.as_deref() {
                validate_graph_input_kind(item_kind, &format!("{path}/items"), diagnostics);
            }
        }
        GraphInputKind::Object { properties } => {
            if let Some(properties) = properties {
                for (property_name, property_spec) in properties {
                    validate_graph_input_spec(
                        property_spec,
                        &format!("{path}/properties/{property_name}"),
                        diagnostics,
                    );
                }
            }
        }
        GraphInputKind::String
        | GraphInputKind::Integer
        | GraphInputKind::Float
        | GraphInputKind::Boolean
        | GraphInputKind::Path => {}
    }

    if let Some(default) = &spec.default {
        if let Err(error) = materialize_graph_input_value(spec, default, &format!("{path}/default"))
        {
            emit_rule(
                diagnostics,
                "E1033",
                error.message,
                error.path,
                Some("Adjust the default value to match the declared input type".to_string()),
            );
        }
    }
}

fn validate_graph_input_kind(
    kind: &GraphInputKind,
    path: &str,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    match kind {
        GraphInputKind::Enum { values } => {
            if values.is_empty() {
                emit_rule(
                    diagnostics,
                    "E1034",
                    "enum input must declare at least one allowed value".to_string(),
                    format!("{path}/values"),
                    Some("Provide one or more enum values".to_string()),
                );
            }
        }
        GraphInputKind::Array { items } => {
            if let Some(item_kind) = items.as_deref() {
                validate_graph_input_kind(item_kind, &format!("{path}/items"), diagnostics);
            }
        }
        GraphInputKind::Object { properties } => {
            if let Some(properties) = properties {
                for (property_name, property_spec) in properties {
                    validate_graph_input_spec(
                        property_spec,
                        &format!("{path}/properties/{property_name}"),
                        diagnostics,
                    );
                }
            }
        }
        GraphInputKind::String
        | GraphInputKind::Integer
        | GraphInputKind::Float
        | GraphInputKind::Boolean
        | GraphInputKind::Path => {}
    }
}

pub fn validate_graph(graph: &Graph) -> Vec<ValidationDiagnostic> {
    graph.validate_with_warnings()
}

pub fn validate_schema(graph: &Graph) -> Vec<ValidationDiagnostic> {
    validate_graph(graph)
        .into_iter()
        .filter(|diag| {
            matches!(classify_rule_domain(diag.code.as_str()), Some(ValidationDomain::Schema))
        })
        .collect()
}

pub fn validate_semantics(graph: &Graph) -> Vec<ValidationDiagnostic> {
    validate_graph(graph)
        .into_iter()
        .filter(|diag| {
            matches!(classify_rule_domain(diag.code.as_str()), Some(ValidationDomain::Semantic))
        })
        .collect()
}

pub fn validate_topology(graph: &Graph) -> Vec<ValidationDiagnostic> {
    validate_graph(graph)
        .into_iter()
        .filter(|diag| {
            matches!(classify_rule_domain(diag.code.as_str()), Some(ValidationDomain::Topology))
        })
        .collect()
}

fn validate_param_value(
    value: &ParamValue,
    graph: &Graph,
    node: &Node,
    order: &HashMap<&str, usize>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
    path: &str,
) {
    match value {
        ParamValue::Ref(spec) => {
            let source_count = usize::from(spec.graph_input.is_some())
                + usize::from(spec.node_output.is_some())
                + usize::from(spec.path_var.is_some());
            if source_count != 1 {
                emit_rule(
                    diagnostics,
                    "E1031",
                    "reference must declare exactly one source".to_string(),
                    path.to_string(),
                    Some("Use exactly one of graph_input, node_output, or path_var".to_string()),
                );
            }
            if let Some(input) = &spec.graph_input {
                if !graph.inputs.contains_key(input) {
                    emit_rule(
                        diagnostics,
                        "E1020",
                        format!("unknown graph input ref: {}", input),
                        path.to_string(),
                        None,
                    );
                }
            }
            if let Some(node_output) = &spec.node_output {
                let target =
                    graph.nodes.iter().find(|candidate| candidate.id == node_output.node_id);
                if target.is_none()
                    || !target
                        .expect("checked above")
                        .outputs
                        .iter()
                        .any(|output| output.name == node_output.output_name)
                {
                    emit_rule(
                        diagnostics,
                        "E1021",
                        format!(
                            "unknown node output ref: {}.{}",
                            node_output.node_id, node_output.output_name
                        ),
                        path.to_string(),
                        None,
                    );
                }
                if let (Some(&source_index), Some(&current_index)) =
                    (order.get(node_output.node_id.as_str()), order.get(node.id.as_str()))
                {
                    if source_index >= current_index {
                        emit_rule(
                            diagnostics,
                            "E1022",
                            format!(
                                "forward node output ref: {}.{}",
                                node_output.node_id, node_output.output_name
                            ),
                            path.to_string(),
                            None,
                        );
                    }
                }
            }
            if let Some(path_var) = &spec.path_var {
                if !is_known_path_variable(path_var.name()) {
                    emit_rule(
                        diagnostics,
                        "E1020",
                        format!("unknown path variable ref: {}", path_var.name()),
                        path.to_string(),
                        Some(
                            "Use one of run_dir, work_dir, inputs_dir, outputs_dir, or cache_dir"
                                .to_string(),
                        ),
                    );
                }
                if let Some(relative_path) = path_var.relative_path() {
                    if !is_valid_relative_path_suffix(relative_path) {
                        emit_rule(
                            diagnostics,
                            "E1025",
                            format!("invalid path variable suffix: {}", relative_path),
                            path.to_string(),
                            Some("Use a normalized relative path without '..'".to_string()),
                        );
                    }
                }
            }
        }
        ParamValue::Array(items) => {
            for (index, value) in items.iter().enumerate() {
                validate_param_value(
                    value,
                    graph,
                    node,
                    order,
                    diagnostics,
                    &format!("{path}/{index}"),
                );
            }
        }
        ParamValue::Object(map) => {
            for (key, value) in map {
                validate_param_value(
                    value,
                    graph,
                    node,
                    order,
                    diagnostics,
                    &format!("{path}/{key}"),
                );
            }
        }
        ParamValue::Literal(_) => {}
    }
}

fn is_valid_relative_path_suffix(path: &str) -> bool {
    is_normalized_relative_path(path) && is_valid_output_path(path)
}

fn classify_rule_domain(code: &str) -> Option<ValidationDomain> {
    validation_rule_registry().iter().find(|rule| rule.id == code).map(|rule| rule.domain)
}
