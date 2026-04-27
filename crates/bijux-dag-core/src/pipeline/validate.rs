use crate::canonical::{error, is_valid_canonical_name, is_valid_output_path, severity_rank, warn};
use crate::{Effect, Graph, GraphError, Node, ParamValue, Severity, ValidationDiagnostic};
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
    ValidationRule { id: "E1005", severity: Severity::Error, domain: ValidationDomain::Schema },
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

impl Graph {
    pub fn validate_with_warnings(&self) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();

        let mut ids = BTreeSet::new();
        let mut node_map: HashMap<&str, &Node> = HashMap::new();
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
                        Some("Use letters, digits, underscores, and optional suffix '*'".to_string()),
                    );
                }
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
                }
            }
            node_map.insert(node.id.as_str(), node);
        }

        let mut edge_pairs = BTreeSet::new();
        let mut target_bindings = BTreeSet::new();
        for edge in &self.edges {
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
                        .any(|output| output.name == node_output.path)
                {
                    emit_rule(
                        diagnostics,
                        "E1021",
                        format!(
                            "unknown node output ref: {}.{}",
                            node_output.node_id, node_output.path
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
                                node_output.node_id, node_output.path
                            ),
                            path.to_string(),
                            None,
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

fn classify_rule_domain(code: &str) -> Option<ValidationDomain> {
    validation_rule_registry().iter().find(|rule| rule.id == code).map(|rule| rule.domain)
}
