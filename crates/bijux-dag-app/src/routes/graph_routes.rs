use crate::commands::DagCli;
use crate::graph::inspection::{build_graph_inspection_payload, GraphInspectionSource};
use crate::graph::selection::{
    selection_summary_for_all_nodes, selection_summary_from_planner,
    selection_summary_from_run_snapshot,
};
use crate::graph_helpers::{
    parse_selectors, resolve_downstream_run_selection, resolve_upstream_run_selection,
    validate_partial_selection_surface,
};
use crate::routes::plan_routes::{build_default_planner_analysis, PlanPreviewConfig};
use crate::routes::preconditions::require_run_directory;
use crate::run_data::load_snapshot;
use crate::{emit_json, load_graphs_or_emit, read_file, ExitCode};
use bijux_dag_runtime::RunSnapshot;
use std::path::{Path, PathBuf};

fn selection_preview_payload(
    graph: &bijux_dag_core::Graph,
    select: &[String],
    exclude: &[String],
    from_node: &[String],
    to_node: &[String],
    dependency_closure: bool,
) -> Result<serde_json::Value, ExitCode> {
    validate_partial_selection_surface(from_node, to_node, select, exclude, dependency_closure)?;
    let selection = if select.is_empty()
        && exclude.is_empty()
        && from_node.is_empty()
        && to_node.is_empty()
        && !dependency_closure
    {
        selection_summary_for_all_nodes(graph)
    } else {
        let selectors = parse_selectors(select, exclude)?;
        let (upstream_selection_targets, _) = resolve_upstream_run_selection(graph, to_node)?;
        let (downstream_selection_roots, _) = resolve_downstream_run_selection(graph, from_node)?;
        let preview = PlanPreviewConfig {
            upstream_selection_targets,
            downstream_selection_roots,
            selectors,
            dependency_closure,
            ..PlanPreviewConfig::default()
        };
        let result =
            build_default_planner_analysis(graph, &preview).map_err(|_| ExitCode::from(3))?;
        selection_summary_from_planner(&result)
    };
    serde_json::to_value(build_graph_inspection_payload(
        graph,
        None,
        GraphInspectionSource { kind: "dag".to_string(), run_dir: None, run_id: None },
        selection,
    ))
    .map_err(|_| ExitCode::from(3))
}

fn run_snapshot_selection_payload(run_dir: &Path) -> Result<serde_json::Value, ExitCode> {
    require_run_directory(run_dir)?;
    let graph_snapshot = load_snapshot(run_dir)?;
    let run_snapshot: RunSnapshot =
        serde_json::from_str(&read_file(&run_dir.join("run.snapshot.json"))?)
            .map_err(|_| ExitCode::from(3))?;
    let selection = selection_summary_from_run_snapshot(&graph_snapshot.graph, &run_snapshot);
    serde_json::to_value(build_graph_inspection_payload(
        &graph_snapshot.graph,
        Some(graph_snapshot.graph_fingerprint),
        GraphInspectionSource {
            kind: "run_dir".to_string(),
            run_dir: Some(run_dir.display().to_string()),
            run_id: Some(run_snapshot.run_id.to_string()),
        },
        selection,
    ))
    .map_err(|_| ExitCode::from(3))
}

pub(crate) fn handle_show_effective_graph_command(
    cli: &DagCli,
    dags: &[PathBuf],
    run_dir: &Option<PathBuf>,
    select: &[String],
    exclude: &[String],
    from_node: &[String],
    to_node: &[String],
    dependency_closure: bool,
) -> Result<ExitCode, ExitCode> {
    let payload = if let Some(run_dir) = run_dir {
        if !dags.is_empty()
            || !select.is_empty()
            || !exclude.is_empty()
            || !from_node.is_empty()
            || !to_node.is_empty()
            || dependency_closure
        {
            return Err(ExitCode::from(2));
        }
        run_snapshot_selection_payload(run_dir)?
    } else {
        let graph = load_graphs_or_emit(cli, "dag.show-effective-graph", dags)?;
        selection_preview_payload(&graph, select, exclude, from_node, to_node, dependency_closure)?
    };

    if cli.json {
        return emit_json(
            cli,
            "dag.show-effective-graph",
            true,
            payload,
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::handle_show_effective_graph_command;
    use crate::commands::DagCli;
    use crate::ExitCode;
    use clap::Parser;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;

    fn quiet_json_cli() -> DagCli {
        DagCli::parse_from(["bijux-dag", "--json", "validate", "graph.json"])
    }

    fn write_graph_fixture() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("graph.json");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "spec":"bijux-dag/v0.1",
                "nodes":[
                    {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
                    {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}},
                    {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":3}}
                ],
                "edges":[
                    {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
                    {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in"}}
                ]
            }))
            .expect("graph"),
        )
        .expect("write graph");
        (dir, path)
    }

    #[test]
    fn show_effective_graph_payload_surfaces_selection_preview() {
        let (_dir, dag) = write_graph_fixture();
        let cli = quiet_json_cli();
        let code = handle_show_effective_graph_command(
            &cli,
            &[dag],
            &None,
            &["id:b".to_string()],
            &[],
            &[],
            &[],
            true,
        )
        .expect("graph inspection");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn show_effective_graph_run_dir_rejects_selection_flags() {
        let cli = quiet_json_cli();
        let error = handle_show_effective_graph_command(
            &cli,
            &[],
            &Some(PathBuf::from("/tmp/run")),
            &["node".to_string()],
            &[],
            &[],
            &[],
            false,
        )
        .expect_err("run-dir mode rejects selector overlay");
        assert_eq!(error, ExitCode::from(2));
    }

    #[test]
    fn selection_preview_payload_contains_selected_nodes() {
        let (_dir, dag) = write_graph_fixture();
        let raw = fs::read_to_string(dag).expect("read graph");
        let graph = crate::parse_graph(&raw).expect("graph");
        let payload =
            super::selection_preview_payload(&graph, &["id:b".to_string()], &[], &[], &[], true)
                .expect("payload");

        let value: Value = payload;
        assert_eq!(value["selection"]["selected_nodes"], serde_json::json!(["a", "b"]));
    }

    #[test]
    fn run_snapshot_payload_surfaces_persisted_selection_and_topology() {
        let dir = tempfile::tempdir().expect("tmp");
        let run_dir = dir.path().join("run-graph");
        fs::create_dir_all(&run_dir).expect("mkdir");
        fs::write(run_dir.join("manifest.json"), br#"{"run_id":"run-graph","status":"success"}"#)
            .expect("manifest");
        fs::write(
            run_dir.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "graph": {
                    "spec":"bijux-dag/v0.1",
                    "nodes":[
                        {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
                        {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}},
                        {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":3}}
                    ],
                    "edges":[
                        {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
                        {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in"}}
                    ]
                },
                "graph_fingerprint":"graph-fp-1"
            }))
            .expect("snapshot"),
        )
        .expect("write graph snapshot");
        fs::write(
            run_dir.join("run.snapshot.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "run_id":"run-graph",
                "graph_snapshot_path":"graph.snapshot.json",
                "planner_config":"default",
                "scheduler_config":"local",
                "policy_config":"runtime-policy-v0.1",
                "provenance":"provenance.json",
                "submission_source":"run",
                "trigger_source":"manual",
                "operator":"operator",
                "labels":[],
                "parent_run_id":null,
                "requested_selectors":["to-node:b"],
                "selected_nodes":["a","b"],
                "dependency_closure_enabled":false,
                "replay_source_run_id":null,
                "partial_rerun_contract":null
            }))
            .expect("run snapshot"),
        )
        .expect("write run snapshot");

        let payload = super::run_snapshot_selection_payload(&run_dir).expect("payload");
        let value: Value = payload;

        assert_eq!(value["source"]["kind"], "run_dir");
        assert_eq!(value["source"]["run_id"], "run-graph");
        assert_eq!(value["graph_fingerprint"], "graph-fp-1");
        assert_eq!(value["selection"]["selected_nodes"], serde_json::json!(["a", "b"]));
        assert_eq!(
            value["selection"]["omitted_nodes"],
            serde_json::json!([{ "node_id": "c", "reason": "omitted_from_run_snapshot" }])
        );
        assert_eq!(value["topology"]["selected_roots"], serde_json::json!(["a"]));
        assert_eq!(value["topology"]["selected_leaves"], serde_json::json!(["b"]));
    }
}
