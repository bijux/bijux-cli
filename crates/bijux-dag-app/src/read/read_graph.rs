use bijux_dag_core::{
    compose_graphs, parse_graph_strict, Graph, GraphCompositionError, GraphError, SPEC_VERSION,
};
use std::fmt;
use std::path::{Path, PathBuf};

pub fn parse_graph(input: &str) -> Result<Graph, GraphError> {
    parse_graph_strict(input)
}

pub fn parse_graph_with_compat(input: &str) -> Result<Graph, GraphError> {
    match parse_graph_strict(input) {
        Ok(g) => Ok(g),
        Err(GraphError::InvalidSpec(_)) => {
            let mut value = serde_json::from_str::<serde_json::Value>(input)?;
            if let Some(spec) = value.get("spec").and_then(serde_json::Value::as_str) {
                if spec == "0.1" || spec == "v0.1" {
                    value["spec"] = serde_json::Value::String(SPEC_VERSION.to_string());
                    let rewritten = serde_json::to_string(&value).map_err(GraphError::from)?;
                    return parse_graph_strict(&rewritten);
                }
            }
            Err(GraphError::InvalidSpec(format!(
                "unsupported spec version: {}",
                value.get("spec").and_then(serde_json::Value::as_str).unwrap_or("<missing>")
            )))
        }
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphLoadError {
    code: u8,
    message: String,
}

impl GraphLoadError {
    pub fn exit_code(&self) -> std::process::ExitCode {
        std::process::ExitCode::from(self.code)
    }

    fn read_failed(path: &Path) -> Self {
        Self { code: 3, message: format!("failed to read graph fragment `{}`", path.display()) }
    }

    fn parse_failed(path: &Path, error: GraphError) -> Self {
        let code = match error {
            GraphError::Json(_) => 2,
            GraphError::InvalidSpec(_) => 1,
            GraphError::ValidationFailed => 3,
        };
        Self {
            code,
            message: format!("failed to parse graph fragment `{}`: {}", path.display(), error),
        }
    }

    fn composition_failed(
        paths: &[PathBuf],
        graphs: &[Graph],
        error: GraphCompositionError,
    ) -> Self {
        Self {
            code: 3,
            message: match &error {
                GraphCompositionError::Empty => error.to_string(),
                GraphCompositionError::UnsupportedSpec { index, .. } => {
                    let path = paths
                        .get(*index)
                        .map(|value| value.display().to_string())
                        .unwrap_or_else(|| "<unknown>".to_string());
                    format!("{error}: {path}")
                }
                GraphCompositionError::ConflictingInputSpec { input_name } => format!(
                    "{error} in {}",
                    format_paths(&paths_for_input_name(paths, graphs, input_name))
                ),
                GraphCompositionError::DuplicateNodeId { node_id } => format!(
                    "{error} in {}",
                    format_paths(&paths_for_node_id(paths, graphs, node_id))
                ),
                GraphCompositionError::DuplicateSubgraphName { subgraph_name } => format!(
                    "{error} in {}",
                    format_paths(&paths_for_subgraph_name(paths, graphs, subgraph_name))
                ),
                GraphCompositionError::DuplicateSubgraphInstanceId { instance_id } => format!(
                    "{error} in {}",
                    format_paths(&paths_for_instance_id(paths, graphs, instance_id))
                ),
            },
        }
    }
}

impl fmt::Display for GraphLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for GraphLoadError {}

pub fn load_graphs(paths: &[PathBuf]) -> Result<Graph, GraphLoadError> {
    if paths.is_empty() {
        return Err(GraphLoadError {
            code: 2,
            message: "graph composition requires at least one graph fragment".to_string(),
        });
    }

    let mut graphs = Vec::with_capacity(paths.len());
    for path in paths {
        let input =
            crate::fs_input::read_utf8_file(path).map_err(|_| GraphLoadError::read_failed(path))?;
        let graph = parse_graph_with_compat(&input)
            .map_err(|error| GraphLoadError::parse_failed(path, error))?;
        graphs.push(graph);
    }

    compose_graphs(&graphs)
        .map_err(|error| GraphLoadError::composition_failed(paths, &graphs, error))
}

fn format_paths(paths: &[String]) -> String {
    paths.iter().map(|path| format!("`{path}`")).collect::<Vec<_>>().join(", ")
}

fn paths_for_input_name(paths: &[PathBuf], graphs: &[Graph], input_name: &str) -> Vec<String> {
    paths_for_fragment(paths, graphs, |graph| graph.inputs.contains_key(input_name))
}

fn paths_for_node_id(paths: &[PathBuf], graphs: &[Graph], node_id: &str) -> Vec<String> {
    paths_for_fragment(paths, graphs, |graph| graph.nodes.iter().any(|node| node.id == node_id))
}

fn paths_for_subgraph_name(
    paths: &[PathBuf],
    graphs: &[Graph],
    subgraph_name: &str,
) -> Vec<String> {
    paths_for_fragment(paths, graphs, |graph| graph.subgraphs.contains_key(subgraph_name))
}

fn paths_for_instance_id(paths: &[PathBuf], graphs: &[Graph], instance_id: &str) -> Vec<String> {
    paths_for_fragment(paths, graphs, |graph| {
        graph.subgraph_instances.iter().any(|instance| instance.id == instance_id)
    })
}

fn paths_for_fragment<F>(paths: &[PathBuf], graphs: &[Graph], predicate: F) -> Vec<String>
where
    F: Fn(&Graph) -> bool,
{
    paths
        .iter()
        .zip(graphs)
        .filter(|(_, graph)| predicate(graph))
        .map(|(path, _)| path.display().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{load_graphs, parse_graph, parse_graph_with_compat};
    use bijux_dag_core::SPEC_VERSION;
    use std::fs;

    #[test]
    fn parses_graph_in_strict_mode() {
        let input = r#"{"spec":"v1","nodes":[],"edges":[]}"#;
        let graph = parse_graph(input).expect("parse graph");
        assert_eq!(graph.spec, SPEC_VERSION);
    }

    #[test]
    fn accepts_legacy_spec_in_compat_mode() {
        let input = r#"{"spec":"0.1","nodes":[],"edges":[]}"#;
        let graph = parse_graph_with_compat(input).expect("parse graph with compat");
        assert_eq!(graph.spec, SPEC_VERSION);
    }

    #[test]
    fn rejects_unknown_spec_in_compat_mode() {
        let input = r#"{"spec":"v9","nodes":[],"edges":[]}"#;
        assert!(parse_graph_with_compat(input).is_err());
    }

    #[test]
    fn loads_and_composes_multiple_graph_fragments() {
        let dir = tempfile::tempdir().expect("tmp");
        let foundation = dir.path().join("foundation.json");
        let publication = dir.path().join("publication.json");
        fs::write(
            &foundation,
            r#"{
              "spec":"bijux-dag/v0.1",
              "inputs":{"region":"eu-west-1"},
              "nodes":[
                {
                  "id":"extract",
                  "kind":"const",
                  "outputs":[{"name":"report","path":"extract/report.json"}],
                  "params":{"value":{"graph_input":"region"}}
                }
              ],
              "edges":[]
            }"#,
        )
        .expect("write foundation");
        fs::write(
            &publication,
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[
                {
                  "id":"publish",
                  "kind":"const",
                  "inputs":["report"],
                  "outputs":[{"name":"out","path":"publish/out.json"}],
                  "params":{"seed":{"node_output":{"node_id":"extract","output_name":"report"}}}
                }
              ],
              "edges":[
                {"from":{"node_id":"extract","port":"report"},"to":{"node_id":"publish","port":"report"}}
              ]
            }"#,
        )
        .expect("write publication");

        let graph =
            load_graphs(&[foundation.clone(), publication.clone()]).expect("load graph fragments");
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(
            graph.resolve_graph().expect("resolve").resolved_params["publish"]["seed"],
            "extract/report.json"
        );
    }

    #[test]
    fn load_graphs_reports_duplicate_node_sources() {
        let dir = tempfile::tempdir().expect("tmp");
        let left = dir.path().join("foundation.json");
        let right = dir.path().join("publication.json");
        let fragment = r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"extract","kind":"const","outputs":[{"name":"out","path":"extract/out"}],"params":{"value":"seed"}}],
          "edges":[]
        }"#;
        fs::write(&left, fragment).expect("write left");
        fs::write(&right, fragment).expect("write right");

        let error = load_graphs(&[left.clone(), right.clone()]).expect_err("duplicate node");
        assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
        assert!(error.to_string().contains("duplicate node id `extract`"));
        assert!(error.to_string().contains(&left.display().to_string()));
        assert!(error.to_string().contains(&right.display().to_string()));
    }
}
