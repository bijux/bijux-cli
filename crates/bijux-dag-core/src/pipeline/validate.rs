use crate::canonical::{error, is_valid_canonical_name, is_valid_output_path, severity_rank, warn};
use crate::{
    EdgeKind, Effect, Graph, GraphError, Node, ParamValue, SemanticNodeKind, Severity, TriggerRule,
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
    ValidationRule { id: "E1028", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1029", severity: Severity::Error, domain: ValidationDomain::Topology },
    ValidationRule { id: "E1030", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1031", severity: Severity::Error, domain: ValidationDomain::Semantic },
    ValidationRule { id: "E1032", severity: Severity::Error, domain: ValidationDomain::Semantic },
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

fn trigger_rule_supports_conditional_incoming(rule: &TriggerRule) -> bool {
    matches!(rule, TriggerRule::AnySuccess | TriggerRule::AllDone)
}

impl Graph {
    pub fn validate_with_warnings(&self) -> Vec<ValidationDiagnostic> {
        let mut diagnostics = Vec::new();

        let mut ids = BTreeSet::new();
        let mut node_map: HashMap<&str, &Node> = HashMap::new();
        let mut branch_output_by_node: HashMap<&str, &str> = HashMap::new();
        let mut branch_decisions_by_node: HashMap<&str, BTreeSet<String>> = HashMap::new();
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
            node_map.insert(node.id.as_str(), node);
        }

        let mut edge_pairs = BTreeSet::new();
        let mut target_bindings = BTreeSet::new();
        let mut conditional_edge_counts = HashMap::<(String, String), usize>::new();
        let mut conditional_incoming_targets = BTreeSet::new();
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
            if spec.graph_input.is_some() == spec.node_output.is_some() {
                emit_rule(
                    diagnostics,
                    "E1031",
                    "reference must declare exactly one source".to_string(),
                    path.to_string(),
                    Some(
                        "Use either graph_input or node_output, and do not provide both"
                            .to_string(),
                    ),
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
