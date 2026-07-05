use crate::commands::DagCli;
use crate::{emit_json, load_graphs_or_emit, ExitCode};
use bijux_dag_core::Severity;
use serde_json::{json, Value};
use std::path::PathBuf;

pub(crate) fn handle_validate_command(
    cli: &DagCli,
    dags: &[PathBuf],
    strict: bool,
    print_fingerprints: bool,
    explain: bool,
) -> Result<ExitCode, ExitCode> {
    let graph = load_graphs_or_emit(cli, "dag.validate", dags)?;
    let diags = graph.validate_with_warnings();
    let has_errors = diags.iter().any(|d| d.severity == Severity::Error);
    let has_warnings = diags.iter().any(|d| d.severity == Severity::Warning);
    let fail = has_errors || (strict && has_warnings);

    let diagnostics: Vec<Value> = diags.iter().map(|d| serde_json::to_value(d).unwrap()).collect();
    let mut data = json!({});
    if print_fingerprints || explain {
        data["graph_fingerprint"] = json!(graph.graph_fingerprint().unwrap());
        let mut nodes = serde_json::Map::new();
        let resolved = graph.resolve_graph().ok().map(|g| g.resolved_params);
        for n in &graph.nodes {
            let fp = resolved
                .as_ref()
                .and_then(|m| m.get(&n.id))
                .and_then(|p| graph.node_fingerprint_with_params(n, p).ok())
                .unwrap_or_else(|| graph.node_fingerprint(n).unwrap());
            nodes.insert(n.id.clone(), json!(fp));
        }
        data["node_fingerprints"] = json!(nodes);
    }
    if explain {
        let canonical = graph.canonicalize();
        let order = canonical.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>();
        data["canonical_order"] = json!(order);
        data["resolved_params"] =
            json!(graph.resolve_graph().map(|g| g.resolved_params).unwrap_or_default());
    }
    if cli.json {
        let code = if fail { ExitCode::from(2) } else { ExitCode::SUCCESS };
        return emit_json(cli, "dag.validate", !fail, data, diagnostics, code);
    } else if !cli.quiet {
        for d in &diags {
            println!("{} {} {}", d.code, d.path, d.message);
        }
        if print_fingerprints {
            println!("graph_fingerprint={}", graph.graph_fingerprint().unwrap());
            let resolved = graph.resolve_graph().ok().map(|g| g.resolved_params);
            for n in &graph.nodes {
                let fp = resolved
                    .as_ref()
                    .and_then(|m| m.get(&n.id))
                    .and_then(|p| graph.node_fingerprint_with_params(n, p).ok())
                    .unwrap_or_else(|| graph.node_fingerprint(n).unwrap());
                println!("node_fingerprint {}={}", n.id, fp);
            }
        }
        if explain {
            let canonical = graph.canonicalize();
            let order = canonical.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>();
            println!("canonical_order: {:?}", order);
            println!(
                "resolved_params: {}",
                serde_json::to_string_pretty(
                    &graph.resolve_graph().map(|g| g.resolved_params).unwrap_or_default()
                )
                .unwrap()
            );
            println!("graph_fingerprint={}", graph.graph_fingerprint().unwrap());
            let resolved = graph.resolve_graph().ok().map(|g| g.resolved_params);
            for n in &graph.nodes {
                let fp = resolved
                    .as_ref()
                    .and_then(|m| m.get(&n.id))
                    .and_then(|p| graph.node_fingerprint_with_params(n, p).ok())
                    .unwrap_or_else(|| graph.node_fingerprint(n).unwrap());
                println!("node_fingerprint {}={}", n.id, fp);
            }
        }
    }
    if !cli.quiet {
        println!("status: {}", if fail { "invalid" } else { "ok" });
    }
    if fail {
        return Err(ExitCode::from(2));
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::handle_validate_command;
    use crate::commands::DagCli;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn validate_route_rejects_missing_file_without_panic() {
        let cli = DagCli::parse_from(["bijux-dag", "validate", "/missing.json"]);
        let code =
            handle_validate_command(&cli, &[PathBuf::from("/missing.json")], false, false, false);
        assert!(code.is_err());
    }

    #[test]
    fn validate_route_accepts_composed_graph_fragments() {
        let dir = tempfile::tempdir().expect("tmp");
        let foundation = dir.path().join("foundation.json");
        let publication = dir.path().join("publication.json");
        std::fs::write(
            &foundation,
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[{"id":"extract","kind":"const","outputs":[{"name":"report","path":"extract/report.json"}],"params":{"value":"seed"}}],
              "edges":[]
            }"#,
        )
        .expect("write foundation");
        std::fs::write(
            &publication,
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[{"id":"publish","kind":"const","inputs":["report"],"outputs":[{"name":"out","path":"publish/out.json"}],"params":{"seed":{"node_output":{"node_id":"extract","output_name":"report"}}}}],
              "edges":[{"from":{"node_id":"extract","port":"report"},"to":{"node_id":"publish","port":"report"}}]
            }"#,
        )
        .expect("write publication");

        let cli = DagCli::parse_from([
            "bijux-dag",
            "--json",
            "validate",
            foundation.to_string_lossy().as_ref(),
            publication.to_string_lossy().as_ref(),
        ]);
        let code = handle_validate_command(&cli, &[foundation, publication], false, false, false)
            .expect("validate");
        assert_eq!(code, std::process::ExitCode::SUCCESS);
    }
}
