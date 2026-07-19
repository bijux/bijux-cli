//! Graph composition across multiple authored fragments.

use crate::{Graph, GraphInputSpec, GraphMeta, SPEC_VERSION};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphCompositionError {
    #[error("graph composition requires at least one graph fragment")]
    Empty,
    #[error("graph composition fragment {index} declared unsupported spec `{found}`; expected `{expected}`")]
    UnsupportedSpec { index: usize, expected: &'static str, found: String },
    #[error("graph composition found conflicting input spec for `{input_name}`")]
    ConflictingInputSpec { input_name: String },
    #[error("graph composition found duplicate node id `{node_id}`")]
    DuplicateNodeId { node_id: String },
    #[error("graph composition found duplicate subgraph name `{subgraph_name}`")]
    DuplicateSubgraphName { subgraph_name: String },
    #[error("graph composition found duplicate subgraph instance id `{instance_id}`")]
    DuplicateSubgraphInstanceId { instance_id: String },
}

pub fn compose_graphs(graphs: &[Graph]) -> Result<Graph, GraphCompositionError> {
    let Some(first) = graphs.first() else {
        return Err(GraphCompositionError::Empty);
    };

    if graphs.len() == 1 {
        return Ok(first.clone());
    }

    let mut inputs = BTreeMap::new();
    let mut subgraphs = BTreeMap::new();
    let mut subgraph_instance_ids = BTreeSet::new();
    let mut subgraph_instances = Vec::new();
    let mut node_ids = BTreeSet::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut nondeterminism_allowed = false;

    for (index, graph) in graphs.iter().enumerate() {
        if graph.spec != SPEC_VERSION {
            return Err(GraphCompositionError::UnsupportedSpec {
                index,
                expected: SPEC_VERSION,
                found: graph.spec.clone(),
            });
        }

        nondeterminism_allowed |= graph.nondeterminism_allowed;

        merge_input_specs(&mut inputs, &graph.inputs)?;

        for (name, definition) in &graph.subgraphs {
            if subgraphs.insert(name.clone(), definition.clone()).is_some() {
                return Err(GraphCompositionError::DuplicateSubgraphName {
                    subgraph_name: name.clone(),
                });
            }
        }

        for instance in &graph.subgraph_instances {
            if !subgraph_instance_ids.insert(instance.id.clone()) {
                return Err(GraphCompositionError::DuplicateSubgraphInstanceId {
                    instance_id: instance.id.clone(),
                });
            }
            subgraph_instances.push(instance.clone());
        }

        for node in &graph.nodes {
            if !node_ids.insert(node.id.clone()) {
                return Err(GraphCompositionError::DuplicateNodeId { node_id: node.id.clone() });
            }
            nodes.push(node.clone());
        }

        edges.extend(graph.edges.clone());
    }

    Ok(Graph {
        spec: SPEC_VERSION.to_string(),
        meta: compose_graph_meta(graphs),
        inputs,
        nondeterminism_allowed,
        subgraphs,
        subgraph_instances,
        nodes,
        edges,
    })
}

fn merge_input_specs(
    merged: &mut BTreeMap<String, GraphInputSpec>,
    fragment_inputs: &BTreeMap<String, GraphInputSpec>,
) -> Result<(), GraphCompositionError> {
    for (input_name, input_spec) in fragment_inputs {
        match merged.get(input_name) {
            Some(existing) if existing != input_spec => {
                return Err(GraphCompositionError::ConflictingInputSpec {
                    input_name: input_name.clone(),
                });
            }
            Some(_) => {}
            None => {
                merged.insert(input_name.clone(), input_spec.clone());
            }
        }
    }
    Ok(())
}

fn compose_graph_meta(graphs: &[Graph]) -> Option<GraphMeta> {
    let metas = graphs.iter().filter_map(|graph| graph.meta.as_ref()).collect::<Vec<_>>();
    if metas.is_empty() {
        return None;
    }

    let names = metas.iter().map(|meta| meta.name.clone()).collect::<BTreeSet<_>>();
    let descriptions = metas.iter().map(|meta| meta.description.clone()).collect::<BTreeSet<_>>();
    let owners = metas
        .iter()
        .flat_map(|meta| meta.owners.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let tags = metas
        .iter()
        .flat_map(|meta| meta.tags.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    Some(GraphMeta {
        name: names.into_iter().collect::<Vec<_>>().join("__"),
        description: if descriptions.len() == 1 {
            descriptions.into_iter().next().flatten()
        } else {
            None
        },
        owners,
        tags,
    })
}

#[cfg(test)]
mod tests {
    use super::compose_graphs;
    use crate::{parse_graph_strict, GraphCompositionError, Severity};
    use serde_json::json;

    fn parse_graph(input: &str) -> crate::Graph {
        parse_graph_strict(input).expect("parse graph")
    }

    #[test]
    fn composition_merges_cross_file_inputs_outputs_and_metadata() {
        let foundation = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"foundation","description":"demo","owners":["ops"],"tags":["base"]},
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
        );
        let publication = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"publication","description":"demo","owners":["bio"],"tags":["publish"]},
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
        );

        let composed = compose_graphs(&[foundation, publication]).expect("compose graphs");
        assert_eq!(
            composed.meta.as_ref().map(|meta| meta.name.as_str()),
            Some("foundation__publication")
        );
        assert_eq!(
            composed.meta.as_ref().map(|meta| meta.description.as_deref()),
            Some(Some("demo"))
        );
        assert_eq!(
            composed.meta.as_ref().map(|meta| meta.owners.clone()),
            Some(vec!["bio".to_string(), "ops".to_string()])
        );
        assert_eq!(
            composed.meta.as_ref().map(|meta| meta.tags.clone()),
            Some(vec!["base".to_string(), "publish".to_string()])
        );
        assert_eq!(composed.inputs["region"].schema_json()["default"], json!("eu-west-1"));
        let diagnostics = composed.validate_with_warnings();
        assert!(
            !diagnostics.iter().any(|diagnostic| diagnostic.severity == Severity::Error),
            "{diagnostics:?}"
        );
        let resolved = composed.resolve_graph().expect("resolve graph");
        assert_eq!(resolved.resolved_params["extract"]["value"], json!("eu-west-1"));
        assert_eq!(resolved.resolved_params["publish"]["seed"], json!("extract/report.json"));
    }

    #[test]
    fn composition_rejects_conflicting_input_specs() {
        let left = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "inputs":{"region":"eu-west-1"},
              "nodes":[],
              "edges":[]
            }"#,
        );
        let right = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "inputs":{"region":{"type":"integer","default":1}},
              "nodes":[],
              "edges":[]
            }"#,
        );

        let error = compose_graphs(&[left, right]).expect_err("composition should fail");
        assert_eq!(
            error,
            GraphCompositionError::ConflictingInputSpec { input_name: "region".to_string() }
        );
    }

    #[test]
    fn composition_rejects_duplicate_node_ids() {
        let left = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[{"id":"extract","kind":"const","outputs":[{"name":"out","path":"extract/out"}],"params":{"value":"left"}}],
              "edges":[]
            }"#,
        );
        let right = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[{"id":"extract","kind":"const","outputs":[{"name":"out","path":"extract-right/out"}],"params":{"value":"right"}}],
              "edges":[]
            }"#,
        );

        let error = compose_graphs(&[left, right]).expect_err("composition should fail");
        assert_eq!(
            error,
            GraphCompositionError::DuplicateNodeId { node_id: "extract".to_string() }
        );
    }

    #[test]
    fn composition_rejects_duplicate_subgraph_names_and_instance_ids() {
        let left = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "subgraphs":{
                "shared":{
                  "graph":{"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
                  "outputs":{}
                }
              },
              "subgraph_instances":[{"id":"reuse","subgraph":"shared"}],
              "nodes":[],
              "edges":[]
            }"#,
        );
        let right_with_duplicate_subgraph = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "subgraphs":{
                "shared":{
                  "graph":{"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
                  "outputs":{}
                }
              },
              "nodes":[],
              "edges":[]
            }"#,
        );
        let right_with_duplicate_instance = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "subgraphs":{
                "other":{
                  "graph":{"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
                  "outputs":{}
                }
              },
              "subgraph_instances":[{"id":"reuse","subgraph":"other"}],
              "nodes":[],
              "edges":[]
            }"#,
        );

        let subgraph_error = compose_graphs(&[left.clone(), right_with_duplicate_subgraph])
            .expect_err("duplicate subgraph");
        assert_eq!(
            subgraph_error,
            GraphCompositionError::DuplicateSubgraphName { subgraph_name: "shared".to_string() }
        );

        let instance_error =
            compose_graphs(&[left, right_with_duplicate_instance]).expect_err("duplicate instance");
        assert_eq!(
            instance_error,
            GraphCompositionError::DuplicateSubgraphInstanceId { instance_id: "reuse".to_string() }
        );
    }

    #[test]
    fn composition_fingerprint_is_independent_of_fragment_order() {
        let left = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"foundation","owners":["ops"],"tags":["base"]},
              "nodes":[{"id":"extract","kind":"const","outputs":[{"name":"out","path":"extract/out"}],"params":{"value":"seed"}}],
              "edges":[]
            }"#,
        );
        let right = parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"publication","owners":["bio"],"tags":["publish"]},
              "nodes":[{"id":"publish","kind":"const","inputs":["seed"],"outputs":[{"name":"out","path":"publish/out"}],"params":{"seed":{"node_output":{"node_id":"extract","output_name":"out"}}}}],
              "edges":[{"from":{"node_id":"extract","port":"out"},"to":{"node_id":"publish","port":"seed"}}]
            }"#,
        );

        let forward = compose_graphs(&[left.clone(), right.clone()]).expect("compose forward");
        let reverse = compose_graphs(&[right, left]).expect("compose reverse");
        assert_eq!(
            forward.graph_fingerprint().expect("forward fingerprint"),
            reverse.graph_fingerprint().expect("reverse fingerprint")
        );
    }
}
