use crate::commands::{DagCli, ReleaseCommands};
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_artifacts::hash::sha256_hex;
use serde::Serialize;

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

pub(crate) fn handle_release_command(
    cli: &DagCli,
    command: &ReleaseCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        ReleaseCommands::Version { dag } => {
            let payload = serde_json::to_value(version_payload(dag)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.release.version", true, payload, Vec::new(), ExitCode::SUCCESS)
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
}
