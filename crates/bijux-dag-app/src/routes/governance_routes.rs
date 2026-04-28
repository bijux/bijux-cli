use crate::commands::{DagCli, GovernanceCommands};
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_artifacts::hash::sha256_hex;
use bijux_dag_core::{compile_graph, node_io_contract, NodeInputSource, Severity};
use bijux_dag_core::node::derive_interface;
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;

const CRITICALITY_TAGS: &[&str] = &["critical", "high", "standard", "low"];
const ENVIRONMENT_TAGS: &[&str] = &["dev", "staging", "prod"];

#[derive(Debug, Serialize)]
struct GovernanceGraphOutput {
    node_id: String,
    output_name: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct GovernanceNodeContract {
    node_id: String,
    kind: String,
    declared_inputs: Vec<String>,
    declared_outputs: Vec<String>,
    declared_params: Vec<String>,
    declared_effects: Vec<String>,
    input_bindings: Vec<serde_json::Value>,
    param_bindings: Vec<serde_json::Value>,
    env_bindings: Vec<String>,
    outputs: Vec<serde_json::Value>,
    unresolved_inputs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OwnershipReport {
    workflow_name: String,
    owners: Vec<String>,
    owner_count: usize,
    criticality: Option<String>,
    escalation_targets: Vec<String>,
    gaps: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TagsReport {
    workflow_name: String,
    graph_tags: Vec<String>,
    normalized_graph_tags: Vec<String>,
    node_tags: BTreeMap<String, Vec<String>>,
    unknown_tags: Vec<String>,
    missing_dimensions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CostNodeEstimate {
    node_id: String,
    cpu_cores: u32,
    memory_gb: f64,
    timeout_ms: u64,
    max_attempts: u32,
    estimated_cost: f64,
}

#[derive(Debug, Serialize)]
struct WorkflowCostReport {
    workflow_name: String,
    estimated_total_cost: f64,
    cpu_core_hour_rate: f64,
    memory_gb_hour_rate: f64,
    estimable_nodes: usize,
    nodes_missing_estimate_inputs: Vec<String>,
    node_estimates: Vec<CostNodeEstimate>,
}

#[derive(Debug, Serialize)]
struct AlertRoutingReport {
    workflow_name: String,
    event: String,
    criticality: Option<String>,
    owners: Vec<String>,
    primary_targets: Vec<String>,
    secondary_targets: Vec<String>,
    escalation_mode: String,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct GovernancePolicyInput {
    require_owners: bool,
    #[serde(default)]
    required_graph_tags: Vec<String>,
    require_node_tags: bool,
    #[serde(default)]
    forbidden_effects: Vec<String>,
    require_retry_for_effectful_nodes: bool,
    require_timeout_for_effectful_nodes: bool,
    max_retry_attempts: Option<u32>,
}

#[derive(Debug, Serialize)]
struct GovernancePolicyReport {
    workflow_name: String,
    violations: Vec<String>,
    checked_nodes: usize,
}

#[derive(Debug, Serialize)]
struct CatalogNodeRecord {
    node_id: String,
    kind: String,
    tags: Vec<String>,
    outputs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CatalogRunRecord {
    run_id: String,
    status: String,
    artifact_count: usize,
}

#[derive(Debug, serde::Deserialize)]
struct AuditEventSimulation {
    actor: String,
    action: String,
    workflow_id: String,
    reason: String,
    unix_ms: u128,
    #[serde(default)]
    targets: Vec<String>,
    #[serde(default)]
    fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct AuditEventRecord {
    event_id: String,
    actor: String,
    action: String,
    workflow_id: String,
    reason: String,
    unix_ms: u128,
    targets: Vec<String>,
    fields: BTreeMap<String, serde_json::Value>,
}

fn load_graph(path: &Path) -> Result<bijux_dag_core::Graph, ExitCode> {
    let input = read_file(path)?;
    parse_graph(&input)
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn graph_name(graph: &bijux_dag_core::Graph) -> String {
    graph.meta
        .as_ref()
        .map(|meta| meta.name.clone())
        .unwrap_or_else(|| "unnamed-workflow".to_string())
}

fn graph_tags(graph: &bijux_dag_core::Graph) -> Vec<String> {
    graph.meta
        .as_ref()
        .map(|meta| meta.tags.clone())
        .unwrap_or_default()
}

fn normalize_tag(tag: &str) -> String {
    tag.trim().to_lowercase().replace([' ', '_'], "-")
}

fn criticality_tag(tags: &[String]) -> Option<String> {
    tags.iter()
        .map(|tag| normalize_tag(tag))
        .find(|tag| CRITICALITY_TAGS.contains(&tag.as_str()))
}

fn environment_tag(tags: &[String]) -> Option<String> {
    tags.iter()
        .map(|tag| normalize_tag(tag))
        .find(|tag| ENVIRONMENT_TAGS.contains(&tag.as_str()))
}

fn governance_contracts_payload(
    dag: &Path,
) -> Result<(serde_json::Value, bool), ExitCode> {
    let graph = load_graph(dag)?;
    let compiled = compile_graph(&graph).map_err(|_| ExitCode::from(3))?;
    let mut node_contracts = Vec::new();
    let mut unresolved_count = 0usize;

    for node in &compiled.normalized_graph.nodes {
        let interface = derive_interface(node);
        let io_contract = node_io_contract(&compiled.normalized_graph, &node.id)
            .ok_or_else(|| ExitCode::from(3))?;
        let unresolved_inputs = io_contract
            .inputs
            .iter()
            .filter_map(|binding| match &binding.source {
                NodeInputSource::Unbound => Some(binding.name.clone()),
                NodeInputSource::UpstreamOutput { .. } => None,
            })
            .collect::<Vec<_>>();
        unresolved_count += unresolved_inputs.len();

        node_contracts.push(GovernanceNodeContract {
            node_id: node.id.clone(),
            kind: node.kind.as_str().to_string(),
            declared_inputs: interface.declared_inputs,
            declared_outputs: interface
                .declared_outputs
                .iter()
                .map(|output| output.name.clone())
                .collect(),
            declared_params: interface.declared_params,
            declared_effects: interface
                .declared_effects
                .iter()
                .map(|effect| format!("{effect:?}").to_lowercase())
                .collect(),
            input_bindings: io_contract
                .inputs
                .iter()
                .map(|binding| match &binding.source {
                    NodeInputSource::UpstreamOutput { node_id, output_name } => {
                        json!({
                            "name": binding.name,
                            "source": "upstream_output",
                            "node_id": node_id,
                            "output_name": output_name,
                        })
                    }
                    NodeInputSource::Unbound => {
                        json!({
                            "name": binding.name,
                            "source": "unbound",
                        })
                    }
                })
                .collect(),
            param_bindings: io_contract
                .param_bindings
                .iter()
                .map(|binding| serde_json::to_value(binding).unwrap())
                .collect(),
            env_bindings: io_contract
                .env_bindings
                .iter()
                .map(|binding| binding.name.clone())
                .collect(),
            outputs: io_contract
                .outputs
                .iter()
                .map(|binding| serde_json::to_value(binding).unwrap())
                .collect(),
            unresolved_inputs,
        });
    }

    let graph_outputs = compiled
        .normalized_graph
        .nodes
        .iter()
        .flat_map(|node| {
            node.outputs.iter().map(|output| GovernanceGraphOutput {
                node_id: node.id.clone(),
                output_name: output.name.clone(),
                path: output.path.clone(),
            })
        })
        .collect::<Vec<_>>();
    let diagnostic_counts = compiled
        .diagnostics
        .iter()
        .fold(BTreeMap::<String, usize>::new(), |mut acc, diagnostic| {
            let key = match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            *acc.entry(key.to_string()).or_insert(0) += 1;
            acc
        });
    let ok = unresolved_count == 0
        && compiled.diagnostics.iter().all(|diagnostic| diagnostic.severity != Severity::Error);
    Ok((
        json!({
            "graph_name": compiled.normalized_graph.meta.as_ref().map(|meta| meta.name.clone()),
            "owners": compiled.normalized_graph.meta.as_ref().map(|meta| meta.owners.clone()).unwrap_or_default(),
            "tags": compiled.normalized_graph.meta.as_ref().map(|meta| meta.tags.clone()).unwrap_or_default(),
            "graph_input_names": compiled.normalized_graph.inputs.keys().cloned().collect::<Vec<_>>(),
            "graph_fingerprint": compiled.graph_fingerprint,
            "topology_order": compiled.plan_hints.deterministic_topology_order,
            "diagnostic_counts": diagnostic_counts,
            "diagnostics": compiled.diagnostics,
            "graph_outputs": graph_outputs,
            "nodes": node_contracts,
            "unresolved_input_count": unresolved_count,
        }),
        ok,
    ))
}

fn ownership_payload(dag: &Path) -> Result<(serde_json::Value, bool), ExitCode> {
    let graph = load_graph(dag)?;
    let owners = graph
        .meta
        .as_ref()
        .map(|meta| meta.owners.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|owner| owner.trim().to_string())
        .filter(|owner| !owner.is_empty())
        .collect::<Vec<_>>();
    let tags = graph_tags(&graph);
    let criticality = criticality_tag(&tags);
    let mut gaps = Vec::new();
    if owners.is_empty() {
        gaps.push("workflow owners are missing".to_string());
    }
    if criticality.as_deref() == Some("critical") && owners.len() < 2 {
        gaps.push("critical workflows require at least two owners".to_string());
    }
    let escalation_targets = owners
        .iter()
        .map(|owner| format!("pager:{owner}"))
        .collect::<Vec<_>>();
    let report = OwnershipReport {
        workflow_name: graph_name(&graph),
        owner_count: owners.len(),
        owners,
        criticality,
        escalation_targets,
        gaps: gaps.clone(),
    };
    Ok((serde_json::to_value(report).map_err(|_| ExitCode::from(3))?, gaps.is_empty()))
}

fn tags_payload(dag: &Path) -> Result<(serde_json::Value, bool), ExitCode> {
    let graph = load_graph(dag)?;
    let graph_tags = graph_tags(&graph);
    let normalized_graph_tags = graph_tags.iter().map(|tag| normalize_tag(tag)).collect::<Vec<_>>();
    let mut node_tags = BTreeMap::new();
    let mut unknown_tags = Vec::new();
    for node in &graph.nodes {
        let normalized = node.tags.iter().map(|tag| normalize_tag(tag)).collect::<Vec<_>>();
        for tag in &normalized {
            if !CRITICALITY_TAGS.contains(&tag.as_str())
                && !ENVIRONMENT_TAGS.contains(&tag.as_str())
                && !["finance", "etl", "analytics", "bioinformatics", "batch", "streaming"]
                    .contains(&tag.as_str())
            {
                unknown_tags.push(tag.clone());
            }
        }
        if !normalized.is_empty() {
            node_tags.insert(node.id.clone(), normalized);
        }
    }
    unknown_tags.sort();
    unknown_tags.dedup();
    let mut missing_dimensions = Vec::new();
    if criticality_tag(&graph_tags).is_none() {
        missing_dimensions.push("criticality".to_string());
    }
    if environment_tag(&graph_tags).is_none() {
        missing_dimensions.push("environment".to_string());
    }
    let report = TagsReport {
        workflow_name: graph_name(&graph),
        graph_tags,
        normalized_graph_tags,
        node_tags,
        unknown_tags: unknown_tags.clone(),
        missing_dimensions: missing_dimensions.clone(),
    };
    Ok((
        serde_json::to_value(report).map_err(|_| ExitCode::from(3))?,
        unknown_tags.is_empty() && missing_dimensions.is_empty(),
    ))
}

fn cost_payload(
    dag: &Path,
    cpu_core_hour_rate: f64,
    memory_gb_hour_rate: f64,
) -> Result<(serde_json::Value, bool), ExitCode> {
    let graph = load_graph(dag)?;
    let mut node_estimates = Vec::new();
    let mut missing = Vec::new();
    let mut total = 0.0_f64;
    for node in &graph.nodes {
        let Some(resources) = &node.resources else {
            missing.push(node.id.clone());
            continue;
        };
        let Some(timeout_ms) = node.timeout_ms else {
            missing.push(node.id.clone());
            continue;
        };
        let attempts = node.retry.max_attempts.max(1);
        let hours = timeout_ms as f64 / 3_600_000.0;
        let memory_gb = resources.mem_mb as f64 / 1024.0;
        let estimate = (resources.cpu as f64 * cpu_core_hour_rate * hours
            + memory_gb * memory_gb_hour_rate * hours)
            * attempts as f64;
        total += estimate;
        node_estimates.push(CostNodeEstimate {
            node_id: node.id.clone(),
            cpu_cores: resources.cpu,
            memory_gb,
            timeout_ms,
            max_attempts: attempts,
            estimated_cost: estimate,
        });
    }
    let report = WorkflowCostReport {
        workflow_name: graph_name(&graph),
        estimated_total_cost: total,
        cpu_core_hour_rate,
        memory_gb_hour_rate,
        estimable_nodes: node_estimates.len(),
        nodes_missing_estimate_inputs: missing.clone(),
        node_estimates,
    };
    Ok((serde_json::to_value(report).map_err(|_| ExitCode::from(3))?, missing.is_empty()))
}

fn alerts_payload(dag: &Path, event: &str) -> Result<(serde_json::Value, bool), ExitCode> {
    let graph = load_graph(dag)?;
    let owners = graph
        .meta
        .as_ref()
        .map(|meta| meta.owners.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|owner| !owner.trim().is_empty())
        .collect::<Vec<_>>();
    let criticality = criticality_tag(&graph_tags(&graph));
    let mut gaps = Vec::new();
    if owners.is_empty() {
        gaps.push("alert routing requires at least one owner".to_string());
    }
    let escalation_mode = match criticality.as_deref() {
        Some("critical") => "page-immediately",
        Some("high") => "page-during-business-hours",
        _ => "ticket-and-email",
    }
    .to_string();
    let primary_targets = owners
        .iter()
        .map(|owner| {
            if escalation_mode.starts_with("page") {
                format!("pager:{owner}")
            } else {
                format!("email:{owner}")
            }
        })
        .collect::<Vec<_>>();
    let secondary_targets = owners
        .iter()
        .map(|owner| format!("slack:{owner}"))
        .collect::<Vec<_>>();
    let report = AlertRoutingReport {
        workflow_name: graph_name(&graph),
        event: event.to_string(),
        criticality,
        owners,
        primary_targets,
        secondary_targets,
        escalation_mode,
        gaps: gaps.clone(),
    };
    Ok((serde_json::to_value(report).map_err(|_| ExitCode::from(3))?, gaps.is_empty()))
}

fn policy_check_payload(
    dag: &Path,
    policy_path: &Path,
) -> Result<(serde_json::Value, bool), ExitCode> {
    let graph = load_graph(dag)?;
    let policy: GovernancePolicyInput = parse_json_file(policy_path)?;
    let owners = graph
        .meta
        .as_ref()
        .map(|meta| meta.owners.clone())
        .unwrap_or_default();
    let graph_tags = graph_tags(&graph).into_iter().map(|tag| normalize_tag(&tag)).collect::<Vec<_>>();
    let forbidden_effects = policy
        .forbidden_effects
        .iter()
        .map(|effect| normalize_tag(effect))
        .collect::<Vec<_>>();
    let mut violations = Vec::new();
    if policy.require_owners && owners.is_empty() {
        violations.push("workflow owners are required".to_string());
    }
    for required in &policy.required_graph_tags {
        let required = normalize_tag(required);
        if !graph_tags.iter().any(|tag| tag == &required) {
            violations.push(format!("missing required graph tag: {required}"));
        }
    }
    for node in &graph.nodes {
        if policy.require_node_tags && node.tags.is_empty() {
            violations.push(format!("node '{}' is missing tags", node.id));
        }
        let normalized_effects = node
            .effects
            .iter()
            .map(|effect| format!("{effect:?}").to_lowercase())
            .collect::<Vec<_>>();
        for effect in &normalized_effects {
            if forbidden_effects.iter().any(|forbidden| forbidden == effect) {
                violations.push(format!("node '{}' uses forbidden effect '{}'", node.id, effect));
            }
        }
        if policy.require_retry_for_effectful_nodes && !node.effects.is_empty() && node.retry.max_attempts == 0 {
            violations.push(format!("effectful node '{}' requires retry policy", node.id));
        }
        if policy.require_timeout_for_effectful_nodes && !node.effects.is_empty() && node.timeout_ms.is_none() {
            violations.push(format!("effectful node '{}' requires timeout_ms", node.id));
        }
        if let Some(max_retry_attempts) = policy.max_retry_attempts {
            if node.retry.max_attempts > max_retry_attempts {
                violations.push(format!(
                    "node '{}' exceeds max retry attempts policy ({})",
                    node.id, max_retry_attempts
                ));
            }
        }
    }
    let report = GovernancePolicyReport {
        workflow_name: graph_name(&graph),
        checked_nodes: graph.nodes.len(),
        violations: violations.clone(),
    };
    Ok((serde_json::to_value(report).map_err(|_| ExitCode::from(3))?, violations.is_empty()))
}

fn catalog_export_payload(
    dag: &Path,
    run_dir: &Option<std::path::PathBuf>,
) -> Result<serde_json::Value, ExitCode> {
    let graph = load_graph(dag)?;
    let compiled = compile_graph(&graph).map_err(|_| ExitCode::from(3))?;
    let nodes = graph
        .nodes
        .iter()
        .map(|node| CatalogNodeRecord {
            node_id: node.id.clone(),
            kind: node.kind.as_str().to_string(),
            tags: node.tags.iter().map(|tag| normalize_tag(tag)).collect(),
            outputs: node.outputs.iter().map(|output| output.path.clone()).collect(),
        })
        .collect::<Vec<_>>();
    let run_record = if let Some(run_dir) = run_dir {
        let manifest: serde_json::Value = parse_json_file(&run_dir.join("manifest.json"))?;
        let outputs_index: serde_json::Value = parse_json_file(&run_dir.join("outputs").join("index.json"))?;
        Some(CatalogRunRecord {
            run_id: manifest.get("run_id").and_then(serde_json::Value::as_str).unwrap_or("unknown-run").to_string(),
            status: manifest.get("status").and_then(serde_json::Value::as_str).unwrap_or("unknown").to_string(),
            artifact_count: outputs_index
                .get("files")
                .and_then(serde_json::Value::as_array)
                .map_or(0, |files| files.len()),
        })
    } else {
        None
    };
    Ok(json!({
        "catalog_format": "dag-catalog/v1",
        "workflow_name": graph_name(&graph),
        "owners": graph.meta.as_ref().map(|meta| meta.owners.clone()).unwrap_or_default(),
        "tags": graph_tags(&graph).into_iter().map(|tag| normalize_tag(&tag)).collect::<Vec<_>>(),
        "graph_fingerprint": compiled.graph_fingerprint,
        "graph_input_names": graph.inputs.keys().cloned().collect::<Vec<_>>(),
        "nodes": nodes,
        "run_record": run_record,
    }))
}

fn audit_event_payload(simulation: &Path) -> Result<(serde_json::Value, bool), ExitCode> {
    let simulation: AuditEventSimulation = parse_json_file(simulation)?;
    let ok = !simulation.actor.trim().is_empty()
        && !simulation.action.trim().is_empty()
        && !simulation.workflow_id.trim().is_empty()
        && !simulation.reason.trim().is_empty();
    let fingerprint_source = serde_json::to_vec(&json!({
        "actor": simulation.actor,
        "action": simulation.action,
        "workflow_id": simulation.workflow_id,
        "reason": simulation.reason,
        "unix_ms": simulation.unix_ms,
        "targets": simulation.targets,
        "fields": simulation.fields,
    }))
    .map_err(|_| ExitCode::from(3))?;
    let event = AuditEventRecord {
        event_id: format!("audit-{}", sha256_hex(&fingerprint_source)),
        actor: simulation.actor,
        action: simulation.action,
        workflow_id: simulation.workflow_id,
        reason: simulation.reason,
        unix_ms: simulation.unix_ms,
        targets: simulation.targets,
        fields: simulation.fields,
    };
    Ok((serde_json::to_value(event).map_err(|_| ExitCode::from(3))?, ok))
}

pub(crate) fn handle_governance_command(
    cli: &DagCli,
    command: &GovernanceCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        GovernanceCommands::Contracts { dag } => {
            let (payload, ok) = governance_contracts_payload(dag)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.governance.contracts",
                    ok,
                    payload,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"governance_contracts_unresolved_inputs",
                            "severity":"error",
                            "message":"graph contract surface contains unresolved inputs or validation errors",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
        }
        GovernanceCommands::Ownership { dag } => {
            let (payload, ok) = ownership_payload(dag)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.governance.ownership",
                    ok,
                    payload,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"governance_ownership_gap",
                            "severity":"error",
                            "message":"workflow ownership coverage is incomplete",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
        }
        GovernanceCommands::Tags { dag } => {
            let (payload, ok) = tags_payload(dag)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.governance.tags",
                    ok,
                    payload,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"governance_tag_taxonomy_gap",
                            "severity":"error",
                            "message":"workflow tags do not satisfy the expected taxonomy",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
        }
        GovernanceCommands::Cost {
            dag,
            cpu_core_hour_rate,
            memory_gb_hour_rate,
        } => {
            let (payload, ok) = cost_payload(dag, *cpu_core_hour_rate, *memory_gb_hour_rate)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.governance.cost",
                    ok,
                    payload,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"governance_cost_missing_estimate_inputs",
                            "severity":"error",
                            "message":"workflow cost estimate is incomplete because some nodes are missing resources or timeouts",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
        }
        GovernanceCommands::Alerts { dag, event } => {
            let (payload, ok) = alerts_payload(dag, event)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.governance.alerts",
                    ok,
                    payload,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"governance_alert_routing_gap",
                            "severity":"error",
                            "message":"workflow alert routing is incomplete because ownership metadata is missing",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
        }
        GovernanceCommands::PolicyCheck { dag, policy } => {
            let (payload, ok) = policy_check_payload(dag, policy)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.governance.policy-check",
                    ok,
                    payload,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"governance_policy_check_failed",
                            "severity":"error",
                            "message":"workflow violates governance policy requirements",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
        }
        GovernanceCommands::CatalogExport { dag, run_dir } => {
            let payload = catalog_export_payload(dag, run_dir)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.governance.catalog-export",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        GovernanceCommands::AuditEvent { simulation } => {
            let (payload, ok) = audit_event_payload(simulation)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.governance.audit-event",
                    ok,
                    payload,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"governance_audit_event_invalid",
                            "severity":"error",
                            "message":"audit event input is missing required identity fields",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
        }
        | GovernanceCommands::Promotion { .. }
        | GovernanceCommands::Compliance { .. } => Err(ExitCode::from(2)),
    }
}

#[cfg(test)]
mod tests {
    use super::handle_governance_command;
    use crate::commands::{Commands, DagCli, GovernanceCommands};
    use crate::ExitCode;
    use clap::Parser;

    fn quiet_json_cli(command: GovernanceCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Governance { command } }
    }

    fn write_valid_graph(path: &std::path::Path) {
        std::fs::write(
            path,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"governance","owners":["platform@bijux","analytics@bijux"],"tags":["critical","prod","finance"]},
              "inputs":{"region":"eu"},
              "nodes":[
                {"id":"extract","kind":"const","inputs":[],"outputs":[{"name":"dataset","path":"extract/dataset.json"}],"params":{"value":"x"}},
                {"id":"score","kind":"const","inputs":["dataset"],"outputs":[{"name":"report","path":"score/report.json"}],"tags":["analytics"],"params":{"region":{"graph_input":"region"}}}
              ],
              "edges":[
                {"from":{"node_id":"extract","port":"dataset"},"to":{"node_id":"score","port":"dataset"}}
              ]
            }"#,
        )
        .expect("graph");
    }

    #[test]
    fn governance_contracts_surface_reports_node_io_contracts() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        write_valid_graph(&dag);
        let cli = quiet_json_cli(GovernanceCommands::Contracts { dag: dag.clone() });
        let code =
            handle_governance_command(&cli, &GovernanceCommands::Contracts { dag }).expect("contracts");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn governance_contracts_surface_rejects_unbound_inputs() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph-bad.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"bad","owners":["platform@bijux"],"tags":["critical"]},
              "nodes":[
                {"id":"score","kind":"shell","inputs":["dataset"],"outputs":[],"params":{"value":"x"}}
              ],
              "edges":[]
            }"#,
        )
        .expect("bad graph");
        let cli = quiet_json_cli(GovernanceCommands::Contracts { dag: dag.clone() });
        let exit = handle_governance_command(&cli, &GovernanceCommands::Contracts { dag })
            .expect_err("unresolved inputs should fail");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn governance_contracts_missing_file_does_not_panic() {
        let cli = DagCli::parse_from([
            "dag",
            "--json",
            "governance",
            "contracts",
            "/missing/file.json",
        ]);
        let result = std::panic::catch_unwind(|| {
            let _ = handle_governance_command(
                &cli,
                &GovernanceCommands::Contracts { dag: "/missing/file.json".into() },
            );
        });
        assert!(result.is_ok());
    }

    #[test]
    fn governance_ownership_surface_accepts_critical_multi_owner_workflow() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        write_valid_graph(&dag);
        let cli = quiet_json_cli(GovernanceCommands::Ownership { dag: dag.clone() });
        let code =
            handle_governance_command(&cli, &GovernanceCommands::Ownership { dag }).expect("ownership");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn governance_ownership_surface_rejects_critical_single_owner_workflow() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"critical","owners":["platform@bijux"],"tags":["critical","prod"]},
              "nodes":[{"id":"extract","kind":"const","inputs":[],"outputs":[],"params":{"value":"x"}}],
              "edges":[]
            }"#,
        )
        .expect("critical graph");
        let cli = quiet_json_cli(GovernanceCommands::Ownership { dag: dag.clone() });
        let exit = handle_governance_command(&cli, &GovernanceCommands::Ownership { dag })
            .expect_err("critical ownership gap should fail");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn governance_tags_surface_accepts_known_taxonomy() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        write_valid_graph(&dag);
        let cli = quiet_json_cli(GovernanceCommands::Tags { dag: dag.clone() });
        let code = handle_governance_command(&cli, &GovernanceCommands::Tags { dag })
            .expect("tags");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn governance_tags_surface_rejects_missing_dimensions_and_unknown_tags() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"untagged","owners":["platform@bijux"],"tags":["Finance Ops"]},
              "nodes":[{"id":"extract","kind":"const","inputs":[],"outputs":[],"tags":["weird_tag"],"params":{"value":"x"}}],
              "edges":[]
            }"#,
        )
        .expect("untagged graph");
        let cli = quiet_json_cli(GovernanceCommands::Tags { dag: dag.clone() });
        let exit = handle_governance_command(&cli, &GovernanceCommands::Tags { dag })
            .expect_err("tag taxonomy should fail");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn governance_cost_surface_estimates_budgeted_workflow_cost() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"costed","owners":["platform@bijux"],"tags":["high","prod","etl"]},
              "nodes":[
                {"id":"extract","kind":"const","inputs":[],"outputs":[],"timeout_ms":600000,"resources":{"cpu":2,"mem_mb":2048},"retry":{"max_attempts":2,"backoff_ms":1000},"params":{"value":"x"}}
              ],
              "edges":[]
            }"#,
        )
        .expect("costed graph");
        let cli = quiet_json_cli(GovernanceCommands::Cost {
            dag: dag.clone(),
            cpu_core_hour_rate: 0.04,
            memory_gb_hour_rate: 0.005,
        });
        let code = handle_governance_command(
            &cli,
            &GovernanceCommands::Cost {
                dag,
                cpu_core_hour_rate: 0.04,
                memory_gb_hour_rate: 0.005,
            },
        )
        .expect("cost");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn governance_alert_surface_routes_critical_failures_to_pager_targets() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        write_valid_graph(&dag);
        let cli = quiet_json_cli(GovernanceCommands::Alerts {
            dag: dag.clone(),
            event: "run_failed".to_string(),
        });
        let code = handle_governance_command(
            &cli,
            &GovernanceCommands::Alerts { dag, event: "run_failed".to_string() },
        )
        .expect("alerts");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn governance_alert_surface_rejects_unowned_workflow() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"orphan","owners":[],"tags":["high","prod"]},
              "nodes":[{"id":"extract","kind":"const","inputs":[],"outputs":[],"params":{"value":"x"}}],
              "edges":[]
            }"#,
        )
        .expect("orphan graph");
        let cli = quiet_json_cli(GovernanceCommands::Alerts {
            dag: dag.clone(),
            event: "run_failed".to_string(),
        });
        let exit = handle_governance_command(
            &cli,
            &GovernanceCommands::Alerts { dag, event: "run_failed".to_string() },
        )
        .expect_err("missing ownership should fail");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn governance_policy_check_surface_accepts_compliant_workflow() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"policy-ok","owners":["platform@bijux"],"tags":["critical","prod","finance"]},
              "nodes":[
                {"id":"extract","kind":"const","inputs":[],"outputs":[],"tags":["etl"],"timeout_ms":60000,"retry":{"max_attempts":1,"backoff_ms":1000},"effects":["filesystem"],"params":{"value":"x"}}
              ],
              "edges":[]
            }"#,
        )
        .expect("policy graph");
        let policy = dir.path().join("policy.json");
        std::fs::write(
            &policy,
            r#"{
              "require_owners": true,
              "required_graph_tags": ["critical", "prod"],
              "require_node_tags": true,
              "forbidden_effects": ["network"],
              "require_retry_for_effectful_nodes": true,
              "require_timeout_for_effectful_nodes": true,
              "max_retry_attempts": 2
            }"#,
        )
        .expect("policy");
        let cli = quiet_json_cli(GovernanceCommands::PolicyCheck {
            dag: dag.clone(),
            policy: policy.clone(),
        });
        let code = handle_governance_command(
            &cli,
            &GovernanceCommands::PolicyCheck { dag, policy },
        )
        .expect("policy check");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn governance_policy_check_surface_rejects_policy_violations() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"policy-bad","owners":[],"tags":["dev"]},
              "nodes":[
                {"id":"extract","kind":"const","inputs":[],"outputs":[],"effects":["network"],"params":{"value":"x"}}
              ],
              "edges":[]
            }"#,
        )
        .expect("bad policy graph");
        let policy = dir.path().join("policy.json");
        std::fs::write(
            &policy,
            r#"{
              "require_owners": true,
              "required_graph_tags": ["critical"],
              "require_node_tags": true,
              "forbidden_effects": ["network"],
              "require_retry_for_effectful_nodes": true,
              "require_timeout_for_effectful_nodes": true,
              "max_retry_attempts": 1
            }"#,
        )
        .expect("policy");
        let cli = quiet_json_cli(GovernanceCommands::PolicyCheck {
            dag: dag.clone(),
            policy: policy.clone(),
        });
        let exit = handle_governance_command(
            &cli,
            &GovernanceCommands::PolicyCheck { dag, policy },
        )
        .expect_err("policy violations should fail");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn governance_catalog_export_surface_emits_external_catalog_payload() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        write_valid_graph(&dag);
        let run_dir = dir.path().join("run-01");
        std::fs::create_dir_all(run_dir.join("outputs")).expect("outputs");
        std::fs::write(
            run_dir.join("manifest.json"),
            r#"{"run_id":"run-01","status":"success"}"#,
        )
        .expect("manifest");
        std::fs::write(
            run_dir.join("outputs").join("index.json"),
            r#"{"files":[{"node_id":"extract","node_fingerprint":"fp","sha256":"abc","path":"nodes/extract/report.json"}]}"#,
        )
        .expect("index");
        let cli = quiet_json_cli(GovernanceCommands::CatalogExport {
            dag: dag.clone(),
            run_dir: Some(run_dir.clone()),
        });
        let code = handle_governance_command(
            &cli,
            &GovernanceCommands::CatalogExport { dag, run_dir: Some(run_dir) },
        )
        .expect("catalog export");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn governance_audit_event_surface_emits_stable_identity_record() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("audit.json");
        std::fs::write(
            &simulation,
            r#"{
              "actor":"ops@bijux",
              "action":"policy_override",
              "workflow_id":"finance-close",
              "reason":"temporary replay unblock",
              "unix_ms":1700000000000,
              "targets":["run-01","node:score"],
              "fields":{"ticket":"OPS-42","scope":"limited"}
            }"#,
        )
        .expect("audit simulation");
        let cli = quiet_json_cli(GovernanceCommands::AuditEvent {
            simulation: simulation.clone(),
        });
        let code = handle_governance_command(
            &cli,
            &GovernanceCommands::AuditEvent { simulation },
        )
        .expect("audit event");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn governance_audit_event_surface_rejects_missing_identity_fields() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("audit.json");
        std::fs::write(
            &simulation,
            r#"{
              "actor":"",
              "action":"policy_override",
              "workflow_id":"finance-close",
              "reason":"",
              "unix_ms":1700000000000
            }"#,
        )
        .expect("audit simulation");
        let cli = quiet_json_cli(GovernanceCommands::AuditEvent {
            simulation: simulation.clone(),
        });
        let exit = handle_governance_command(
            &cli,
            &GovernanceCommands::AuditEvent { simulation },
        )
        .expect_err("invalid audit event should fail");
        assert_eq!(exit, ExitCode::from(3));
    }
}
