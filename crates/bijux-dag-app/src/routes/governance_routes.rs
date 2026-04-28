use crate::commands::{DagCli, GovernanceCommands};
use crate::{emit_json, parse_graph, read_file, ExitCode};
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

fn load_graph(path: &Path) -> Result<bijux_dag_core::Graph, ExitCode> {
    let input = read_file(path)?;
    parse_graph(&input)
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
        GovernanceCommands::Cost { .. }
        | GovernanceCommands::Alerts { .. }
        | GovernanceCommands::PolicyCheck { .. }
        | GovernanceCommands::CatalogExport { .. }
        | GovernanceCommands::AuditEvent { .. }
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
}
