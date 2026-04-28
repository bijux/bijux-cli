use crate::commands::{DagCli, ReleaseCommands};
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_artifacts::hash::sha256_hex;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;

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

fn load_json_file<T: DeserializeOwned>(path: &std::path::Path) -> Result<T, ExitCode> {
    let raw = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(2))
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

    if simulation.test_results.iter().all(|check| check.passed) && !simulation.test_results.is_empty() {
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

pub(crate) fn handle_release_command(
    cli: &DagCli,
    command: &ReleaseCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        ReleaseCommands::Version { dag } => {
            let payload = serde_json::to_value(version_payload(dag)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.version", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Promotion { simulation } => {
            let payload =
                serde_json::to_value(promotion_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.promotion", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        ReleaseCommands::Deprecation { simulation } => {
            let payload = serde_json::to_value(deprecation_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.deprecation", true, payload, Vec::new(), ExitCode::SUCCESS)
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
        let code =
            handle_release_command(&cli, &ReleaseCommands::Version { dag: dag.clone() }).expect("version");
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
        assert!(
            gaps.iter()
                .any(|v| v.as_str() == Some("workflow revision has no release taxonomy tags"))
        );
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
        for expected in ["tests", "simulations", "approvals", "rollback", "classification", "shadow", "canary"] {
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
        assert!(
            report.gaps.iter().any(|gap| gap == "deprecation must be announced through at least two channels")
        );
        assert!(report.gaps.iter().any(|gap| gap == "deprecation is missing migration guidance"));
    }
}
