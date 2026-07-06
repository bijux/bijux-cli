use crate::commands::DagCli;
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
use serde_json::json;
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
            selectors,
            upstream_selection_targets,
            downstream_selection_roots,
            dependency_closure,
            ..PlanPreviewConfig::default()
        };
        let result =
            build_default_planner_analysis(graph, &preview).map_err(|_| ExitCode::from(3))?;
        selection_summary_from_planner(&result)
    };
    Ok(json!({
        "source": { "kind": "dag" },
        "graph": graph.canonicalize(),
        "selection": selection,
    }))
}

fn run_snapshot_selection_payload(run_dir: &Path) -> Result<serde_json::Value, ExitCode> {
    require_run_directory(run_dir)?;
    let graph_snapshot = load_snapshot(run_dir)?;
    let run_snapshot: RunSnapshot =
        serde_json::from_str(&read_file(&run_dir.join("run.snapshot.json"))?)
            .map_err(|_| ExitCode::from(3))?;
    let selection = selection_summary_from_run_snapshot(&graph_snapshot.graph, &run_snapshot);
    Ok(json!({
        "source": {
            "kind": "run_dir",
            "run_dir": run_dir.display().to_string(),
            "run_id": run_snapshot.run_id.to_string(),
        },
        "graph_fingerprint": graph_snapshot.graph_fingerprint,
        "graph": graph_snapshot.graph.canonicalize(),
        "selection": selection,
    }))
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
        DagCli::parse_from(["bijux-dag", "--json", "validate", "placeholder.json"])
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
}
