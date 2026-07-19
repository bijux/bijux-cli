use crate::commands::{DagCli, ReleaseCommands};
use crate::routes::simulation_io::load_json_file;
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_artifacts::hash::sha256_hex;
use bijux_dag_core::{Graph, Node};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
struct WorkflowRevisionReport {
    workflow_name: String,
    graph_id: String,
    canonical_sha256: String,
    node_count: usize,
    edge_count: usize,
    owners: Vec<String>,
    tags: Vec<String>,
    release_ready: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReleasePromotionSimulation {
    revision_id: String,
    from_environment: String,
    to_environment: String,
    #[serde(default)]
    test_results: Vec<ReleaseEvidenceCheck>,
    #[serde(default)]
    simulation_results: Vec<ReleaseEvidenceCheck>,
    #[serde(default)]
    approval_ids: Vec<String>,
    rollback_revision_id: Option<String>,
    change_classification: String,
    shadow_consistent: bool,
    canary_ready: bool,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseEvidenceCheck {
    name: String,
    passed: bool,
}

#[derive(Debug, Serialize)]
struct ReleasePromotionReport {
    revision_id: String,
    from_environment: String,
    to_environment: String,
    ready: bool,
    required_gates: Vec<String>,
    satisfied_gates: Vec<String>,
    unmet_gates: Vec<String>,
    rollback_revision_id: Option<String>,
    evidence_count: usize,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseDeprecationSimulation {
    surface: String,
    replacement: String,
    warn_after_unix_ms: u128,
    remove_after_unix_ms: u128,
    cli_notice: bool,
    api_notice: bool,
    docs_notice: bool,
    migration_guide: bool,
}

#[derive(Debug, Serialize)]
struct ReleaseDeprecationReport {
    surface: String,
    replacement: String,
    notice_channels: Vec<String>,
    lifecycle_valid: bool,
    ready: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseCheckpointSimulation {
    action: String,
    run_paused: bool,
    approval_required: bool,
    #[serde(default)]
    approvers: Vec<String>,
    #[serde(default)]
    context_fields: Vec<String>,
    audit_recorded: bool,
    resume_actor: Option<String>,
}

#[derive(Debug, Serialize)]
struct ReleaseCheckpointReport {
    action: String,
    approval_state: String,
    ready: bool,
    blockers: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseShadowSimulation {
    baseline_revision_id: String,
    candidate_revision_id: String,
    compare_plan: bool,
    compare_outcomes: bool,
    side_effect_free: bool,
    differing_nodes: usize,
    critical_drift_nodes: usize,
}

#[derive(Debug, Serialize)]
struct ReleaseShadowReport {
    baseline_revision_id: String,
    candidate_revision_id: String,
    comparable: bool,
    drift_class: String,
    warnings: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseCanarySimulation {
    target: String,
    scope_percent: u8,
    max_workflows: usize,
    max_tenants: usize,
    abort_on_error_rate: f64,
    current_error_rate: f64,
    abort_on_sla_breach_rate: f64,
    current_sla_breach_rate: f64,
}

#[derive(Debug, Serialize)]
struct ReleaseCanaryReport {
    target: String,
    within_scope: bool,
    healthy: bool,
    recommendation: String,
    blockers: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseRollbackSimulation {
    current_revision_id: String,
    rollback_revision_id: String,
    rollback_artifacts_ready: bool,
    rollback_policy_ready: bool,
    replay_safe: bool,
    operators_assigned: usize,
    estimated_recovery_minutes: u32,
}

#[derive(Debug, Serialize)]
struct ReleaseRollbackReport {
    current_revision_id: String,
    rollback_revision_id: String,
    guaranteed: bool,
    blockers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReleaseClassificationReport {
    compatibility_class: String,
    added_nodes: Vec<String>,
    removed_nodes: Vec<String>,
    changed_nodes: BTreeMap<String, Vec<String>>,
    graph_policy_changed: bool,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseEvidenceSimulation {
    revision_id: String,
    #[serde(default)]
    tests: Vec<String>,
    #[serde(default)]
    simulations: Vec<String>,
    #[serde(default)]
    prior_runs: Vec<String>,
    #[serde(default)]
    approvals: Vec<String>,
    #[serde(default)]
    artifacts: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReleaseEvidenceReport {
    revision_id: String,
    complete: bool,
    evidence_refs: Vec<String>,
    missing_dimensions: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseHealthSimulation {
    revision_id: String,
    success_rate: f64,
    error_rate: f64,
    sla_breach_rate: f64,
    rollback_triggered: bool,
    page_count: u32,
    canary_scope_percent: u8,
}

#[derive(Debug, Serialize)]
struct ReleaseHealthReport {
    revision_id: String,
    score: i32,
    status: String,
    reasons: Vec<String>,
}

fn version_payload(dag: &std::path::Path) -> Result<WorkflowRevisionReport, ExitCode> {
    let input = read_file(dag)?;
    let graph = parse_graph(&input)?;
    let fingerprint = graph.graph_fingerprint_explain().map_err(|_| ExitCode::from(3))?;
    let canonical_bytes = graph.canonical_json_bytes().map_err(|_| ExitCode::from(3))?;
    let meta = graph.meta.clone();
    let workflow_name = meta
        .as_ref()
        .map(|m| m.name.clone())
        .unwrap_or_else(|| fingerprint.graph_id.as_str().to_string());
    let owners = meta.as_ref().map(|m| m.owners.clone()).unwrap_or_default();
    let tags = meta.as_ref().map(|m| m.tags.clone()).unwrap_or_default();

    let mut gaps = Vec::new();
    if owners.is_empty() {
        gaps.push("workflow revision has no owners".to_string());
    }
    if tags.is_empty() {
        gaps.push("workflow revision has no release taxonomy tags".to_string());
    }
    if graph.nodes.is_empty() {
        gaps.push("workflow revision has no executable nodes".to_string());
    }

    Ok(WorkflowRevisionReport {
        workflow_name,
        graph_id: fingerprint.graph_id.as_str().to_string(),
        canonical_sha256: sha256_hex(&canonical_bytes),
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
        owners,
        tags,
        release_ready: gaps.is_empty(),
        gaps,
    })
}

fn promotion_payload(simulation: &std::path::Path) -> Result<ReleasePromotionReport, ExitCode> {
    let simulation: ReleasePromotionSimulation = load_json_file(simulation)?;
    let required_gates = vec![
        "tests".to_string(),
        "simulations".to_string(),
        "approvals".to_string(),
        "rollback".to_string(),
        "classification".to_string(),
        "shadow".to_string(),
        "canary".to_string(),
    ];
    let mut satisfied_gates = Vec::new();
    let mut unmet_gates = Vec::new();

    if simulation.test_results.iter().all(|check| check.passed)
        && !simulation.test_results.is_empty()
    {
        satisfied_gates.push("tests".to_string());
    } else {
        unmet_gates.push("tests".to_string());
    }
    if simulation.simulation_results.iter().all(|check| check.passed)
        && !simulation.simulation_results.is_empty()
    {
        satisfied_gates.push("simulations".to_string());
    } else {
        unmet_gates.push("simulations".to_string());
    }
    if !simulation.approval_ids.is_empty() {
        satisfied_gates.push("approvals".to_string());
    } else {
        unmet_gates.push("approvals".to_string());
    }
    if simulation.rollback_revision_id.as_ref().is_some_and(|value| !value.trim().is_empty()) {
        satisfied_gates.push("rollback".to_string());
    } else {
        unmet_gates.push("rollback".to_string());
    }
    if matches!(
        simulation.change_classification.as_str(),
        "additive" | "reviewed-risky" | "compatible"
    ) {
        satisfied_gates.push("classification".to_string());
    } else {
        unmet_gates.push("classification".to_string());
    }
    if simulation.shadow_consistent {
        satisfied_gates.push("shadow".to_string());
    } else {
        unmet_gates.push("shadow".to_string());
    }
    if simulation.canary_ready {
        satisfied_gates.push("canary".to_string());
    } else {
        unmet_gates.push("canary".to_string());
    }

    Ok(ReleasePromotionReport {
        revision_id: simulation.revision_id,
        from_environment: simulation.from_environment,
        to_environment: simulation.to_environment,
        ready: unmet_gates.is_empty(),
        required_gates,
        satisfied_gates,
        unmet_gates,
        rollback_revision_id: simulation.rollback_revision_id,
        evidence_count: simulation.test_results.len()
            + simulation.simulation_results.len()
            + simulation.approval_ids.len(),
    })
}

fn deprecation_payload(simulation: &std::path::Path) -> Result<ReleaseDeprecationReport, ExitCode> {
    let simulation: ReleaseDeprecationSimulation = load_json_file(simulation)?;
    let mut notice_channels = Vec::new();
    if simulation.cli_notice {
        notice_channels.push("cli".to_string());
    }
    if simulation.api_notice {
        notice_channels.push("api".to_string());
    }
    if simulation.docs_notice {
        notice_channels.push("docs".to_string());
    }
    let lifecycle_valid = simulation.warn_after_unix_ms < simulation.remove_after_unix_ms;
    let mut gaps = Vec::new();
    if !lifecycle_valid {
        gaps.push("deprecation window is not ordered".to_string());
    }
    if notice_channels.len() < 2 {
        gaps.push("deprecation must be announced through at least two channels".to_string());
    }
    if !simulation.migration_guide {
        gaps.push("deprecation is missing migration guidance".to_string());
    }
    Ok(ReleaseDeprecationReport {
        surface: simulation.surface,
        replacement: simulation.replacement,
        notice_channels,
        lifecycle_valid,
        ready: gaps.is_empty(),
        gaps,
    })
}

fn checkpoint_payload(simulation: &std::path::Path) -> Result<ReleaseCheckpointReport, ExitCode> {
    let simulation: ReleaseCheckpointSimulation = load_json_file(simulation)?;
    let mut blockers = Vec::new();
    if simulation.approval_required && simulation.approvers.is_empty() {
        blockers.push("approval checkpoint has no approvers".to_string());
    }
    if simulation.approval_required && !simulation.run_paused {
        blockers.push("approval-required action is not paused".to_string());
    }
    if simulation.context_fields.len() < 2 {
        blockers.push("approval checkpoint context is too thin".to_string());
    }
    if !simulation.audit_recorded {
        blockers.push("approval checkpoint is missing audit evidence".to_string());
    }
    if simulation.resume_actor.is_none() {
        blockers.push("approval checkpoint has no resume actor".to_string());
    }
    let approval_state = if blockers.is_empty() {
        "ready_to_resume".to_string()
    } else if simulation.run_paused {
        "awaiting_approval".to_string()
    } else {
        "unsafe".to_string()
    };
    Ok(ReleaseCheckpointReport {
        action: simulation.action,
        approval_state,
        ready: blockers.is_empty(),
        blockers,
    })
}

fn shadow_payload(simulation: &std::path::Path) -> Result<ReleaseShadowReport, ExitCode> {
    let simulation: ReleaseShadowSimulation = load_json_file(simulation)?;
    let mut warnings = Vec::new();
    if !simulation.compare_plan {
        warnings.push("shadow run does not compare planner output".to_string());
    }
    if !simulation.compare_outcomes {
        warnings.push("shadow run does not compare execution outcomes".to_string());
    }
    if !simulation.side_effect_free {
        warnings.push("shadow run is not isolated from side effects".to_string());
    }
    if simulation.critical_drift_nodes > 0 {
        warnings.push("shadow run reports critical drift".to_string());
    }
    let comparable =
        warnings.is_empty() || (warnings.len() == 1 && simulation.differing_nodes == 0);
    let drift_class = if simulation.critical_drift_nodes > 0 {
        "critical".to_string()
    } else if simulation.differing_nodes > 0 {
        "observable".to_string()
    } else {
        "clean".to_string()
    };
    Ok(ReleaseShadowReport {
        baseline_revision_id: simulation.baseline_revision_id,
        candidate_revision_id: simulation.candidate_revision_id,
        comparable,
        drift_class,
        warnings,
    })
}

fn canary_payload(simulation: &std::path::Path) -> Result<ReleaseCanaryReport, ExitCode> {
    let simulation: ReleaseCanarySimulation = load_json_file(simulation)?;
    let mut blockers = Vec::new();
    if simulation.scope_percent == 0 || simulation.scope_percent > 25 {
        blockers.push("canary scope must stay between 1 and 25 percent".to_string());
    }
    if simulation.max_workflows == 0 {
        blockers.push("canary must limit workflow count".to_string());
    }
    if simulation.max_tenants == 0 {
        blockers.push("canary must limit tenant count".to_string());
    }
    if simulation.current_error_rate > simulation.abort_on_error_rate {
        blockers.push("error rate exceeds canary abort threshold".to_string());
    }
    if simulation.current_sla_breach_rate > simulation.abort_on_sla_breach_rate {
        blockers.push("sla breach rate exceeds canary abort threshold".to_string());
    }
    let within_scope = blockers.iter().all(|b| {
        b != "canary scope must stay between 1 and 25 percent"
            && b != "canary must limit workflow count"
            && b != "canary must limit tenant count"
    });
    let healthy = !blockers.iter().any(|b| b.contains("exceeds"));
    let recommendation = if blockers.is_empty() {
        "hold_canary_scope".to_string()
    } else if healthy {
        "narrow_scope_before_rollout".to_string()
    } else {
        "abort_canary".to_string()
    };
    Ok(ReleaseCanaryReport {
        target: simulation.target,
        within_scope,
        healthy,
        recommendation,
        blockers,
    })
}

fn rollback_payload(simulation: &std::path::Path) -> Result<ReleaseRollbackReport, ExitCode> {
    let simulation: ReleaseRollbackSimulation = load_json_file(simulation)?;
    let mut blockers = Vec::new();
    if !simulation.rollback_artifacts_ready {
        blockers.push("rollback artifacts are not available".to_string());
    }
    if !simulation.rollback_policy_ready {
        blockers.push("rollback policy is not ready".to_string());
    }
    if !simulation.replay_safe {
        blockers.push("rollback path is not replay-safe".to_string());
    }
    if simulation.operators_assigned == 0 {
        blockers.push("rollback path has no assigned operators".to_string());
    }
    if simulation.estimated_recovery_minutes > 30 {
        blockers.push("rollback recovery time exceeds thirty minutes".to_string());
    }
    Ok(ReleaseRollbackReport {
        current_revision_id: simulation.current_revision_id,
        rollback_revision_id: simulation.rollback_revision_id,
        guaranteed: blockers.is_empty(),
        blockers,
    })
}

fn node_signature(node: &Node) -> Vec<String> {
    let mut signature = Vec::new();
    signature.push(format!("kind:{}", node.kind.as_str()));
    signature.push(format!("inputs:{:?}", node.inputs));
    let outputs = node
        .outputs
        .iter()
        .map(|output| format!("{}->{}", output.name, output.path))
        .collect::<Vec<_>>();
    signature.push(format!("outputs:{outputs:?}"));
    signature.push(format!("params:{:?}", node.params));
    signature.push(format!("timeout:{:?}", node.timeout_ms));
    signature.push(format!("resources:{:?}", node.resources.as_ref().map(|r| (r.cpu, r.mem_mb))));
    signature.push(format!("retry:{}:{}", node.retry.max_attempts, node.retry.backoff_ms));
    signature.push(format!("cache:{}:{:?}", node.cache.enabled, node.cache.reason));
    let mut effects =
        node.effects.iter().map(|effect| format!("{effect:?}").to_lowercase()).collect::<Vec<_>>();
    effects.sort();
    signature.push(format!("effects:{effects:?}"));
    let mut env_allowlist = node.env_allowlist.clone();
    env_allowlist.sort();
    signature.push(format!("env_allowlist:{env_allowlist:?}"));
    let container_signature = node.container.as_ref().map(|container| {
        let mut env_allowlist = container.env_allowlist.clone();
        env_allowlist.sort();
        (
            container.image.clone(),
            container.argv.clone(),
            env_allowlist,
            container.workdir.clone(),
            container.engine.clone(),
        )
    });
    signature.push(format!("container:{container_signature:?}"));
    signature
}

fn classify_payload(
    before: &std::path::Path,
    after: &std::path::Path,
) -> Result<ReleaseClassificationReport, ExitCode> {
    let before_graph: Graph = parse_graph(&read_file(before)?)?;
    let after_graph: Graph = parse_graph(&read_file(after)?)?;

    let before_nodes =
        before_graph.nodes.iter().map(|node| (node.id.clone(), node)).collect::<BTreeMap<_, _>>();
    let after_nodes =
        after_graph.nodes.iter().map(|node| (node.id.clone(), node)).collect::<BTreeMap<_, _>>();

    let mut added_nodes = Vec::new();
    let mut removed_nodes = Vec::new();
    let mut changed_nodes = BTreeMap::new();

    for node_id in after_nodes.keys() {
        if !before_nodes.contains_key(node_id) {
            added_nodes.push(node_id.clone());
        }
    }
    for node_id in before_nodes.keys() {
        if !after_nodes.contains_key(node_id) {
            removed_nodes.push(node_id.clone());
        }
    }
    for (node_id, before_node) in &before_nodes {
        if let Some(after_node) = after_nodes.get(node_id) {
            let before_signature = node_signature(before_node);
            let after_signature = node_signature(after_node);
            if before_signature != after_signature {
                let mut changes = Vec::new();
                for (before_entry, after_entry) in
                    before_signature.iter().zip(after_signature.iter())
                {
                    if before_entry != after_entry {
                        changes.push(format!("{before_entry} -> {after_entry}"));
                    }
                }
                changed_nodes.insert(node_id.clone(), changes);
            }
        }
    }

    let graph_policy_changed = before_graph.meta.as_ref().map(|m| (&m.owners, &m.tags))
        != after_graph.meta.as_ref().map(|m| (&m.owners, &m.tags));

    let compatibility_class = if !removed_nodes.is_empty()
        || changed_nodes.values().flatten().any(|change| {
            change.starts_with("kind:")
                || change.starts_with("outputs:")
                || change.starts_with("inputs:")
        }) {
        "breaking".to_string()
    } else if !changed_nodes.is_empty() || graph_policy_changed {
        "risky".to_string()
    } else if !added_nodes.is_empty() {
        "additive".to_string()
    } else {
        "compatible".to_string()
    };

    Ok(ReleaseClassificationReport {
        compatibility_class,
        added_nodes,
        removed_nodes,
        changed_nodes,
        graph_policy_changed,
    })
}

fn evidence_payload(simulation: &std::path::Path) -> Result<ReleaseEvidenceReport, ExitCode> {
    let simulation: ReleaseEvidenceSimulation = load_json_file(simulation)?;
    let mut evidence_refs = Vec::new();
    evidence_refs.extend(simulation.tests.iter().map(|value| format!("test:{value}")));
    evidence_refs.extend(simulation.simulations.iter().map(|value| format!("simulation:{value}")));
    evidence_refs.extend(simulation.prior_runs.iter().map(|value| format!("run:{value}")));
    evidence_refs.extend(simulation.approvals.iter().map(|value| format!("approval:{value}")));
    evidence_refs.extend(simulation.artifacts.iter().map(|value| format!("artifact:{value}")));

    let mut missing_dimensions = Vec::new();
    if simulation.tests.is_empty() {
        missing_dimensions.push("tests".to_string());
    }
    if simulation.simulations.is_empty() {
        missing_dimensions.push("simulations".to_string());
    }
    if simulation.prior_runs.is_empty() {
        missing_dimensions.push("prior_runs".to_string());
    }
    if simulation.approvals.is_empty() {
        missing_dimensions.push("approvals".to_string());
    }
    if simulation.artifacts.is_empty() {
        missing_dimensions.push("artifacts".to_string());
    }

    Ok(ReleaseEvidenceReport {
        revision_id: simulation.revision_id,
        complete: missing_dimensions.is_empty(),
        evidence_refs,
        missing_dimensions,
    })
}

fn health_payload(simulation: &std::path::Path) -> Result<ReleaseHealthReport, ExitCode> {
    let simulation: ReleaseHealthSimulation = load_json_file(simulation)?;
    let mut score = 100_i32;
    let mut reasons = Vec::new();

    if simulation.success_rate < 0.99 {
        score -= 15;
        reasons.push("success rate is below ninety-nine percent".to_string());
    }
    if simulation.error_rate > 0.02 {
        score -= 25;
        reasons.push("error rate exceeds release threshold".to_string());
    }
    if simulation.sla_breach_rate > 0.01 {
        score -= 20;
        reasons.push("sla breach rate exceeds release threshold".to_string());
    }
    if simulation.rollback_triggered {
        score -= 30;
        reasons.push("rollback trigger fired".to_string());
    }
    if simulation.page_count > 3 {
        score -= 10;
        reasons.push("operator paging load is elevated".to_string());
    }
    if simulation.canary_scope_percent > 25 {
        score -= 10;
        reasons.push("canary scope exceeded guarded rollout size".to_string());
    }

    let status = if score >= 85 {
        "healthy"
    } else if score >= 65 {
        "watch"
    } else {
        "hold_or_rollback"
    }
    .to_string();

    Ok(ReleaseHealthReport { revision_id: simulation.revision_id, score, status, reasons })
}

pub(crate) fn handle_release_command(
    cli: &DagCli,
    command: &ReleaseCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        ReleaseCommands::Version { dag } => {
            let payload =
                serde_json::to_value(version_payload(dag)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.version", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Promotion { simulation } => {
            let payload = serde_json::to_value(promotion_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.promotion", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Deprecation { simulation } => {
            let payload = serde_json::to_value(deprecation_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.deprecation", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Checkpoint { simulation } => {
            let payload = serde_json::to_value(checkpoint_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.checkpoint", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Shadow { simulation } => {
            let payload =
                serde_json::to_value(shadow_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.shadow", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Canary { simulation } => {
            let payload =
                serde_json::to_value(canary_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.canary", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Rollback { simulation } => {
            let payload = serde_json::to_value(rollback_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.rollback", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Classify { before, after } => {
            let payload = serde_json::to_value(classify_payload(before, after)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.classify", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Evidence { simulation } => {
            let payload = serde_json::to_value(evidence_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.evidence", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Health { simulation } => {
            let payload =
                serde_json::to_value(health_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.health", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_release_command;
    use crate::commands::{Commands, DagCli, ReleaseCommands};
    use crate::ExitCode;
    use serde_json::Value;

    fn quiet_json_cli(command: ReleaseCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Release { command } }
    }

    #[test]
    fn release_version_reports_revision_identity() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let dag = dir.path().join("dag.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"releaseable","owners":["team-core"],"tags":["prod","critical"]},
              "nodes":[
                {
                  "id":"extract",
                  "kind":"shell",
                  "inputs":[],
                  "outputs":[{"name":"out","path":"out"}],
                  "params":{"argv":["/bin/sh","-c","echo ok > ../outputs/out"]},
                  "effects":["filesystem"]
                }
              ],
              "edges":[]
            }"#,
        )
        .expect("write dag");
        let cli = quiet_json_cli(ReleaseCommands::Version { dag: dag.clone() });
        let code = handle_release_command(&cli, &ReleaseCommands::Version { dag: dag.clone() })
            .expect("version");
        assert_eq!(code, ExitCode::SUCCESS);
        let payload = super::version_payload(&dag).expect("payload");
        assert!(payload.release_ready);
        assert_eq!(payload.workflow_name, "releaseable");
        assert_eq!(payload.node_count, 1);
        assert_eq!(payload.edge_count, 0);
        assert_eq!(payload.owners, vec!["team-core".to_string()]);
        assert_eq!(payload.tags, vec!["prod".to_string(), "critical".to_string()]);
        assert!(!payload.graph_id.is_empty());
        assert_eq!(payload.canonical_sha256.len(), 64);
    }

    #[test]
    fn release_version_flags_missing_release_metadata() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let dag = dir.path().join("dag.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"anonymous","owners":[],"tags":[]},
              "nodes":[{"id":"n","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"x"}}],
              "edges":[]
            }"#,
        )
        .expect("write dag");
        let payload = super::version_payload(&dag).expect("payload");
        let payload_json = serde_json::to_value(&payload).expect("json");
        assert_eq!(payload_json["release_ready"], Value::Bool(false));
        let gaps = payload_json["gaps"].as_array().expect("gaps");
        assert!(gaps.iter().any(|v| v.as_str() == Some("workflow revision has no owners")));
        assert!(gaps
            .iter()
            .any(|v| v.as_str() == Some("workflow revision has no release taxonomy tags")));
    }

    #[test]
    fn release_promotion_accepts_evidence_backed_revision() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("promotion.json");
        std::fs::write(
            &simulation,
            r#"{
              "revision_id":"graph:abc123",
              "from_environment":"staging",
              "to_environment":"prod",
              "test_results":[{"name":"contracts","passed":true},{"name":"smoke","passed":true}],
              "simulation_results":[{"name":"backfill","passed":true}],
              "approval_ids":["chg-1001"],
              "rollback_revision_id":"graph:prev999",
              "change_classification":"reviewed-risky",
              "shadow_consistent":true,
              "canary_ready":true
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(ReleaseCommands::Promotion { simulation: simulation.clone() });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Promotion { simulation: simulation.clone() },
        )
        .expect("promotion");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::promotion_payload(&simulation).expect("report");
        assert!(report.ready);
        assert!(report.unmet_gates.is_empty());
        assert_eq!(report.evidence_count, 4);
    }

    #[test]
    fn release_promotion_blocks_missing_evidence_and_rollback() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("promotion.json");
        std::fs::write(
            &simulation,
            r#"{
              "revision_id":"graph:broken",
              "from_environment":"staging",
              "to_environment":"prod",
              "test_results":[{"name":"contracts","passed":false}],
              "simulation_results":[],
              "approval_ids":[],
              "rollback_revision_id":null,
              "change_classification":"breaking",
              "shadow_consistent":false,
              "canary_ready":false
            }"#,
        )
        .expect("write simulation");
        let report = super::promotion_payload(&simulation).expect("report");
        assert!(!report.ready);
        for expected in
            ["tests", "simulations", "approvals", "rollback", "classification", "shadow", "canary"]
        {
            assert!(report.unmet_gates.iter().any(|gate| gate == expected));
        }
    }

    #[test]
    fn release_deprecation_requires_ordered_lifecycle_and_guidance() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("deprecation.json");
        std::fs::write(
            &simulation,
            r#"{
              "surface":"dag.run.legacy-mode",
              "replacement":"dag.run.hermetic",
              "warn_after_unix_ms":100,
              "remove_after_unix_ms":200,
              "cli_notice":true,
              "api_notice":false,
              "docs_notice":true,
              "migration_guide":true
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(ReleaseCommands::Deprecation { simulation: simulation.clone() });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Deprecation { simulation: simulation.clone() },
        )
        .expect("deprecation");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::deprecation_payload(&simulation).expect("report");
        assert!(report.ready);
        assert_eq!(report.notice_channels, vec!["cli".to_string(), "docs".to_string()]);
    }

    #[test]
    fn release_deprecation_rejects_missing_notices() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("deprecation.json");
        std::fs::write(
            &simulation,
            r#"{
              "surface":"dag.run.legacy-mode",
              "replacement":"dag.run.hermetic",
              "warn_after_unix_ms":300,
              "remove_after_unix_ms":200,
              "cli_notice":true,
              "api_notice":false,
              "docs_notice":false,
              "migration_guide":false
            }"#,
        )
        .expect("write simulation");
        let report = super::deprecation_payload(&simulation).expect("report");
        assert!(!report.ready);
        assert!(report.gaps.iter().any(|gap| gap == "deprecation window is not ordered"));
        assert!(report
            .gaps
            .iter()
            .any(|gap| gap == "deprecation must be announced through at least two channels"));
        assert!(report.gaps.iter().any(|gap| gap == "deprecation is missing migration guidance"));
    }

    #[test]
    fn release_checkpoint_accepts_audited_resume_gate() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("checkpoint.json");
        std::fs::write(
            &simulation,
            r#"{
              "action":"publish-results",
              "run_paused":true,
              "approval_required":true,
              "approvers":["owner-a","owner-b"],
              "context_fields":["lineage","destination"],
              "audit_recorded":true,
              "resume_actor":"owner-a"
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(ReleaseCommands::Checkpoint { simulation: simulation.clone() });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Checkpoint { simulation: simulation.clone() },
        )
        .expect("checkpoint");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::checkpoint_payload(&simulation).expect("report");
        assert!(report.ready);
        assert_eq!(report.approval_state, "ready_to_resume");
    }

    #[test]
    fn release_checkpoint_blocks_unpaused_and_unowned_gate() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("checkpoint.json");
        std::fs::write(
            &simulation,
            r#"{
              "action":"publish-results",
              "run_paused":false,
              "approval_required":true,
              "approvers":[],
              "context_fields":["lineage"],
              "audit_recorded":false,
              "resume_actor":null
            }"#,
        )
        .expect("write simulation");
        let report = super::checkpoint_payload(&simulation).expect("report");
        assert!(!report.ready);
        for expected in [
            "approval checkpoint has no approvers",
            "approval-required action is not paused",
            "approval checkpoint context is too thin",
            "approval checkpoint is missing audit evidence",
            "approval checkpoint has no resume actor",
        ] {
            assert!(report.blockers.iter().any(|blocker| blocker == expected));
        }
    }

    #[test]
    fn release_shadow_accepts_clean_side_effect_free_comparison() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("shadow.json");
        std::fs::write(
            &simulation,
            r#"{
              "baseline_revision_id":"graph:stable",
              "candidate_revision_id":"graph:candidate",
              "compare_plan":true,
              "compare_outcomes":true,
              "side_effect_free":true,
              "differing_nodes":0,
              "critical_drift_nodes":0
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(ReleaseCommands::Shadow { simulation: simulation.clone() });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Shadow { simulation: simulation.clone() },
        )
        .expect("shadow");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::shadow_payload(&simulation).expect("report");
        assert!(report.comparable);
        assert_eq!(report.drift_class, "clean");
    }

    #[test]
    fn release_shadow_flags_missing_comparisons_and_critical_drift() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("shadow.json");
        std::fs::write(
            &simulation,
            r#"{
              "baseline_revision_id":"graph:stable",
              "candidate_revision_id":"graph:candidate",
              "compare_plan":false,
              "compare_outcomes":false,
              "side_effect_free":false,
              "differing_nodes":4,
              "critical_drift_nodes":2
            }"#,
        )
        .expect("write simulation");
        let report = super::shadow_payload(&simulation).expect("report");
        assert!(!report.comparable);
        assert_eq!(report.drift_class, "critical");
        for expected in [
            "shadow run does not compare planner output",
            "shadow run does not compare execution outcomes",
            "shadow run is not isolated from side effects",
            "shadow run reports critical drift",
        ] {
            assert!(report.warnings.iter().any(|warning| warning == expected));
        }
    }

    #[test]
    fn release_canary_accepts_bounded_healthy_rollout() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("canary.json");
        std::fs::write(
            &simulation,
            r#"{
              "target":"scheduler-policy-v2",
              "scope_percent":10,
              "max_workflows":5,
              "max_tenants":2,
              "abort_on_error_rate":0.05,
              "current_error_rate":0.01,
              "abort_on_sla_breach_rate":0.02,
              "current_sla_breach_rate":0.0
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(ReleaseCommands::Canary { simulation: simulation.clone() });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Canary { simulation: simulation.clone() },
        )
        .expect("canary");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::canary_payload(&simulation).expect("report");
        assert!(report.within_scope);
        assert!(report.healthy);
        assert_eq!(report.recommendation, "hold_canary_scope");
    }

    #[test]
    fn release_canary_blocks_wide_and_unhealthy_rollout() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("canary.json");
        std::fs::write(
            &simulation,
            r#"{
              "target":"scheduler-policy-v2",
              "scope_percent":40,
              "max_workflows":0,
              "max_tenants":0,
              "abort_on_error_rate":0.05,
              "current_error_rate":0.2,
              "abort_on_sla_breach_rate":0.02,
              "current_sla_breach_rate":0.1
            }"#,
        )
        .expect("write simulation");
        let report = super::canary_payload(&simulation).expect("report");
        assert!(!report.within_scope);
        assert!(!report.healthy);
        assert_eq!(report.recommendation, "abort_canary");
        for expected in [
            "canary scope must stay between 1 and 25 percent",
            "canary must limit workflow count",
            "canary must limit tenant count",
            "error rate exceeds canary abort threshold",
            "sla breach rate exceeds canary abort threshold",
        ] {
            assert!(report.blockers.iter().any(|blocker| blocker == expected));
        }
    }

    #[test]
    fn release_rollback_accepts_prepared_recovery_path() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("rollback.json");
        std::fs::write(
            &simulation,
            r#"{
              "current_revision_id":"graph:new",
              "rollback_revision_id":"graph:old",
              "rollback_artifacts_ready":true,
              "rollback_policy_ready":true,
              "replay_safe":true,
              "operators_assigned":2,
              "estimated_recovery_minutes":15
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(ReleaseCommands::Rollback { simulation: simulation.clone() });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Rollback { simulation: simulation.clone() },
        )
        .expect("rollback");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::rollback_payload(&simulation).expect("report");
        assert!(report.guaranteed);
    }

    #[test]
    fn release_rollback_flags_missing_recovery_requirements() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("rollback.json");
        std::fs::write(
            &simulation,
            r#"{
              "current_revision_id":"graph:new",
              "rollback_revision_id":"graph:old",
              "rollback_artifacts_ready":false,
              "rollback_policy_ready":false,
              "replay_safe":false,
              "operators_assigned":0,
              "estimated_recovery_minutes":45
            }"#,
        )
        .expect("write simulation");
        let report = super::rollback_payload(&simulation).expect("report");
        assert!(!report.guaranteed);
        for expected in [
            "rollback artifacts are not available",
            "rollback policy is not ready",
            "rollback path is not replay-safe",
            "rollback path has no assigned operators",
            "rollback recovery time exceeds thirty minutes",
        ] {
            assert!(report.blockers.iter().any(|blocker| blocker == expected));
        }
    }

    #[test]
    fn release_classification_marks_additive_change() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let before = dir.path().join("before.json");
        let after = dir.path().join("after.json");
        std::fs::write(
            &before,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":["team"],"tags":["prod"]},
              "nodes":[{"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"x"}}],
              "edges":[]
            }"#,
        )
        .expect("write before");
        std::fs::write(
            &after,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":["team"],"tags":["prod"]},
              "nodes":[
                {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"x"}},
                {"id":"b","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"y"}}
              ],
              "edges":[]
            }"#,
        )
        .expect("write after");
        let cli = quiet_json_cli(ReleaseCommands::Classify {
            before: before.clone(),
            after: after.clone(),
        });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Classify { before: before.clone(), after: after.clone() },
        )
        .expect("classify");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::classify_payload(&before, &after).expect("report");
        assert_eq!(report.compatibility_class, "additive");
        assert_eq!(report.added_nodes, vec!["b".to_string()]);
    }

    #[test]
    fn release_classification_marks_breaking_contract_change() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let before = dir.path().join("before.json");
        let after = dir.path().join("after.json");
        std::fs::write(
            &before,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":["team"],"tags":["prod"]},
              "nodes":[{"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"x"}}],
              "edges":[]
            }"#,
        )
        .expect("write before");
        std::fs::write(
            &after,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":["team"],"tags":["prod"]},
              "nodes":[{"id":"a","kind":"shell","inputs":["in"],"outputs":[{"name":"other","path":"other"}],"params":{"argv":["/bin/true"]}}],
              "edges":[]
            }"#,
        )
        .expect("write after");
        let report = super::classify_payload(&before, &after).expect("report");
        assert_eq!(report.compatibility_class, "breaking");
        assert!(report.changed_nodes.contains_key("a"));
    }

    #[test]
    fn release_classification_tracks_cache_and_env_contract_changes() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let before = dir.path().join("before.json");
        let after = dir.path().join("after.json");
        std::fs::write(
            &before,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":["team"],"tags":["prod"]},
              "nodes":[{
                "id":"publish",
                "kind":"shell",
                "inputs":[],
                "outputs":[{"name":"report","path":"report.json"}],
                "params":{"argv":["/bin/true"]},
                "cache":{"enabled":true},
                "effects":["env","filesystem"],
                "env_allowlist":["REPORT_TOKEN"]
              }],
              "edges":[]
            }"#,
        )
        .expect("write before");
        std::fs::write(
            &after,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":["team"],"tags":["prod"]},
              "nodes":[{
                "id":"publish",
                "kind":"shell",
                "inputs":[],
                "outputs":[{"name":"report","path":"report.json"}],
                "params":{"argv":["/bin/true"]},
                "cache":{"enabled":false,"reason":"publishes externally visible state"},
                "effects":["env","filesystem"],
                "env_allowlist":["REPORT_TOKEN","REPORT_CHANNEL"]
              }],
              "edges":[]
            }"#,
        )
        .expect("write after");

        let report = super::classify_payload(&before, &after).expect("report");
        assert_eq!(report.compatibility_class, "risky");
        let changes = report.changed_nodes.get("publish").expect("publish changes");
        assert!(changes.iter().any(|change| change.starts_with("cache:")));
        assert!(changes.iter().any(|change| change.starts_with("env_allowlist:")));
    }

    #[test]
    fn release_classification_tracks_param_reference_changes() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let before = dir.path().join("before.json");
        let after = dir.path().join("after.json");
        std::fs::write(
            &before,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":["team"],"tags":["prod"]},
              "inputs":{"region":"eu","dataset_uri":"s3://warehouse/catalog"},
              "nodes":[
                {
                  "id":"extract",
                  "kind":"const",
                  "inputs":[],
                  "outputs":[{"name":"request","path":"request.json"}],
                  "params":{
                    "value":{
                      "dataset_uri":{"graph_input":"dataset_uri"},
                      "region":{"graph_input":"region"}
                    }
                  }
                },
                {
                  "id":"publish",
                  "kind":"shell",
                  "inputs":["request"],
                  "outputs":[{"name":"report","path":"report.json"}],
                  "params":{
                    "report_source":{"node_output":{"node_id":"extract","output_name":"request"}},
                    "argv":["/bin/true"]
                  }
                }
              ],
              "edges":[{"from":{"node_id":"extract","port":"request"},"to":{"node_id":"publish","port":"request"}}]
            }"#,
        )
        .expect("write before");
        std::fs::write(
            &after,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"wf","owners":["team"],"tags":["prod"]},
              "inputs":{"region":"eu","dataset_uri":"s3://warehouse/catalog","tenant":"atlas"},
              "nodes":[
                {
                  "id":"extract",
                  "kind":"const",
                  "inputs":[],
                  "outputs":[{"name":"request","path":"request.json"}],
                  "params":{
                    "value":{
                      "dataset_uri":{"graph_input":"dataset_uri"},
                      "tenant":{"graph_input":"tenant"}
                    }
                  }
                },
                {
                  "id":"publish",
                  "kind":"shell",
                  "inputs":["request"],
                  "outputs":[{"name":"report","path":"report.json"}],
                  "params":{
                    "report_source":{"node_output":{"node_id":"extract","output_name":"report"}},
                    "argv":["/bin/true"]
                  }
                }
              ],
              "edges":[{"from":{"node_id":"extract","port":"request"},"to":{"node_id":"publish","port":"request"}}]
            }"#,
        )
        .expect("write after");

        let report = super::classify_payload(&before, &after).expect("report");
        assert_eq!(report.compatibility_class, "risky");
        let extract_changes = report.changed_nodes.get("extract").expect("extract changes");
        assert!(extract_changes.iter().any(|change| change.starts_with("params:")));
        let publish_changes = report.changed_nodes.get("publish").expect("publish changes");
        assert!(publish_changes.iter().any(|change| change.starts_with("params:")));
    }

    #[test]
    fn release_evidence_links_promotion_inputs() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("evidence.json");
        std::fs::write(
            &simulation,
            r#"{
              "revision_id":"graph:new",
              "tests":["contracts","smoke"],
              "simulations":["shadow-clean"],
              "prior_runs":["run-1001"],
              "approvals":["chg-1"],
              "artifacts":["bundle-1"]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(ReleaseCommands::Evidence { simulation: simulation.clone() });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Evidence { simulation: simulation.clone() },
        )
        .expect("evidence");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::evidence_payload(&simulation).expect("report");
        assert!(report.complete);
        assert!(report.evidence_refs.iter().any(|value| value == "test:contracts"));
        assert!(report.evidence_refs.iter().any(|value| value == "run:run-1001"));
    }

    #[test]
    fn release_evidence_reports_missing_dimensions() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("evidence.json");
        std::fs::write(
            &simulation,
            r#"{
              "revision_id":"graph:new",
              "tests":[],
              "simulations":[],
              "prior_runs":[],
              "approvals":[],
              "artifacts":[]
            }"#,
        )
        .expect("write simulation");
        let report = super::evidence_payload(&simulation).expect("report");
        assert!(!report.complete);
        for expected in ["tests", "simulations", "prior_runs", "approvals", "artifacts"] {
            assert!(report.missing_dimensions.iter().any(|value| value == expected));
        }
    }

    #[test]
    fn release_health_scores_stable_revision_as_healthy() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("health.json");
        std::fs::write(
            &simulation,
            r#"{
              "revision_id":"graph:new",
              "success_rate":0.995,
              "error_rate":0.005,
              "sla_breach_rate":0.0,
              "rollback_triggered":false,
              "page_count":1,
              "canary_scope_percent":10
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(ReleaseCommands::Health { simulation: simulation.clone() });
        let code = handle_release_command(
            &cli,
            &ReleaseCommands::Health { simulation: simulation.clone() },
        )
        .expect("health");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::health_payload(&simulation).expect("report");
        assert_eq!(report.status, "healthy");
        assert!(report.score >= 85);
    }

    #[test]
    fn release_health_flags_revision_for_hold_or_rollback() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("health.json");
        std::fs::write(
            &simulation,
            r#"{
              "revision_id":"graph:new",
              "success_rate":0.8,
              "error_rate":0.1,
              "sla_breach_rate":0.05,
              "rollback_triggered":true,
              "page_count":5,
              "canary_scope_percent":40
            }"#,
        )
        .expect("write simulation");
        let report = super::health_payload(&simulation).expect("report");
        assert_eq!(report.status, "hold_or_rollback");
        assert!(report.score < 65);
        assert!(report.reasons.iter().any(|reason| reason == "rollback trigger fired"));
    }
}
