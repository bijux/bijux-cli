use crate::{Edge, Graph, Node, NodeOutputRef, ParamValue, RefSpec};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const DYNAMIC_EXPANSION_SCHEMA_VERSION: &str = "bijux-dag-dynamic-expansion/v0.1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DynamicSpec {
    pub expansion_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicExpansionDocument {
    #[serde(default = "default_dynamic_expansion_schema_version")]
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DynamicExpansionRecord {
    pub controller_node_id: String,
    pub expansion_output: String,
    pub expansion_fingerprint: String,
    pub generated_node_ids: Vec<String>,
    pub generated_edge_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedDynamicExpansion {
    pub graph: Graph,
    pub record: DynamicExpansionRecord,
}

fn default_dynamic_expansion_schema_version() -> String {
    DYNAMIC_EXPANSION_SCHEMA_VERSION.to_string()
}

pub fn generated_node_id(controller_node_id: &str, raw_node_id: &str) -> String {
    format!("{controller_node_id}__{raw_node_id}")
}

pub fn parse_dynamic_expansion_document(
    raw: &str,
) -> Result<DynamicExpansionDocument, String> {
    let document: DynamicExpansionDocument =
        serde_json::from_str(raw).map_err(|error| format!("invalid dynamic expansion document: {error}"))?;
    if document.schema_version != DYNAMIC_EXPANSION_SCHEMA_VERSION {
        return Err(format!(
            "unsupported dynamic expansion schema version '{}'",
            document.schema_version
        ));
    }
    Ok(document)
}

pub fn apply_dynamic_expansion(
    graph: &Graph,
    controller_node_id: &str,
    document: DynamicExpansionDocument,
) -> Result<AppliedDynamicExpansion, String> {
    if document.schema_version != DYNAMIC_EXPANSION_SCHEMA_VERSION {
        return Err(format!(
            "unsupported dynamic expansion schema version '{}'",
            document.schema_version
        ));
    }

    let controller = graph
        .nodes
        .iter()
        .find(|node| node.id == controller_node_id)
        .ok_or_else(|| format!("dynamic controller '{}' is not present in the graph", controller_node_id))?;
    let dynamic = controller.dynamic.as_ref().ok_or_else(|| {
        format!("dynamic controller '{}' is missing its dynamic contract", controller_node_id)
    })?;

    let base_edge_conflict = graph.edges.iter().any(|edge| {
        edge.from.node_id == controller_node_id || edge.to.node_id == controller_node_id
    });
    if base_edge_conflict {
        return Err(format!(
            "dynamic controller '{}' must not have declared graph edges; generated fragments own their connectivity",
            controller_node_id
        ));
    }

    let raw_node_ids =
        document.nodes.iter().map(|node| node.id.clone()).collect::<BTreeSet<String>>();
    if raw_node_ids.len() != document.nodes.len() {
        return Err(format!(
            "dynamic controller '{}' produced duplicate generated node ids",
            controller_node_id
        ));
    }

    let mut rewritten_ids = BTreeMap::new();
    for raw_node_id in &raw_node_ids {
        rewritten_ids.insert(
            raw_node_id.clone(),
            generated_node_id(controller_node_id, raw_node_id),
        );
    }

    let mut existing_ids = graph
        .nodes
        .iter()
        .filter(|node| node.id != controller_node_id)
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    for rewritten in rewritten_ids.values() {
        if !existing_ids.insert(rewritten.clone()) {
            return Err(format!(
                "dynamic controller '{}' would generate duplicate node id '{}'",
                controller_node_id, rewritten
            ));
        }
    }

    let mut generated_nodes = Vec::with_capacity(document.nodes.len());
    for node in document.nodes {
        let mut rewritten = node;
        rewritten.id = rewritten_ids
            .get(&rewritten.id)
            .cloned()
            .ok_or_else(|| "generated node rewrite map is incomplete".to_string())?;
        rewrite_param_value_refs(&mut rewritten.params, &raw_node_ids, &rewritten_ids, &existing_ids)?;
        generated_nodes.push(rewritten);
    }

    let mut generated_edges = Vec::with_capacity(document.edges.len());
    for edge in document.edges {
        let mut rewritten = edge;
        rewritten.from.node_id =
            rewrite_endpoint_id(&rewritten.from.node_id, &raw_node_ids, &rewritten_ids);
        rewritten.to.node_id =
            rewrite_endpoint_id(&rewritten.to.node_id, &raw_node_ids, &rewritten_ids);
        if !existing_ids.contains(&rewritten.from.node_id) {
            return Err(format!(
                "dynamic controller '{}' generated edge from unknown node '{}'",
                controller_node_id, rewritten.from.node_id
            ));
        }
        if !existing_ids.contains(&rewritten.to.node_id) {
            return Err(format!(
                "dynamic controller '{}' generated edge to unknown node '{}'",
                controller_node_id, rewritten.to.node_id
            ));
        }
        generated_edges.push(rewritten);
    }

    let generated_node_ids = generated_nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let expansion_fingerprint = dynamic_expansion_fingerprint(&generated_nodes, &generated_edges)?;

    let mut expanded = graph.clone();
    expanded.nodes.retain(|node| node.id != controller_node_id);
    expanded.edges.retain(|edge| {
        edge.from.node_id != controller_node_id && edge.to.node_id != controller_node_id
    });
    expanded.nodes.extend(generated_nodes);
    expanded.edges.extend(generated_edges.clone());

    Ok(AppliedDynamicExpansion {
        graph: expanded,
        record: DynamicExpansionRecord {
            controller_node_id: controller_node_id.to_string(),
            expansion_output: dynamic.expansion_output.clone(),
            expansion_fingerprint,
            generated_node_ids,
            generated_edge_count: generated_edges.len(),
        },
    })
}

fn rewrite_endpoint_id(
    candidate: &str,
    raw_node_ids: &BTreeSet<String>,
    rewritten_ids: &BTreeMap<String, String>,
) -> String {
    if raw_node_ids.contains(candidate) {
        rewritten_ids.get(candidate).cloned().unwrap_or_else(|| candidate.to_string())
    } else {
        candidate.to_string()
    }
}

fn rewrite_param_value_refs(
    value: &mut ParamValue,
    raw_node_ids: &BTreeSet<String>,
    rewritten_ids: &BTreeMap<String, String>,
    known_node_ids: &BTreeSet<String>,
) -> Result<(), String> {
    match value {
        ParamValue::Ref(reference) => rewrite_ref_spec(reference, raw_node_ids, rewritten_ids, known_node_ids),
        ParamValue::Array(items) => {
            for item in items {
                rewrite_param_value_refs(item, raw_node_ids, rewritten_ids, known_node_ids)?;
            }
            Ok(())
        }
        ParamValue::Object(map) => {
            for item in map.values_mut() {
                rewrite_param_value_refs(item, raw_node_ids, rewritten_ids, known_node_ids)?;
            }
            Ok(())
        }
        ParamValue::Literal(_) => Ok(()),
    }
}

fn rewrite_ref_spec(
    reference: &mut RefSpec,
    raw_node_ids: &BTreeSet<String>,
    rewritten_ids: &BTreeMap<String, String>,
    known_node_ids: &BTreeSet<String>,
) -> Result<(), String> {
    if let Some(NodeOutputRef { node_id, .. }) = reference.node_output.as_mut() {
        if raw_node_ids.contains(node_id) {
            *node_id = rewritten_ids
                .get(node_id)
                .cloned()
                .ok_or_else(|| "generated node rewrite map is incomplete".to_string())?;
        } else if !known_node_ids.contains(node_id) {
            return Err(format!(
                "generated node output reference points to unknown node '{}'",
                node_id
            ));
        }
    }
    Ok(())
}

fn dynamic_expansion_fingerprint(nodes: &[Node], edges: &[Edge]) -> Result<String, String> {
    let payload = serde_json::to_vec(&(nodes, edges))
        .map_err(|error| format!("failed to serialize dynamic expansion payload: {error}"))?;
    let mut hasher = Sha256::new();
    hasher.update(payload);
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::{
        apply_dynamic_expansion, generated_node_id, parse_dynamic_expansion_document,
        DynamicExpansionDocument, DYNAMIC_EXPANSION_SCHEMA_VERSION,
    };
    use crate::{parse_graph_strict, DynamicExpansionRecord, Edge, NodeOutputRef, ParamValue};
    use serde_json::json;

    fn base_graph() -> crate::Graph {
        parse_graph_strict(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[
                {
                  "id":"seed",
                  "kind":"const",
                  "outputs":[{"name":"out","path":"seed/out.json","kind":"value"}],
                  "params":{"value":["north","south"]}
                },
                {
                  "id":"expand_regions",
                  "kind":"const",
                  "semantic_kind":"dynamic",
                  "outputs":[{"name":"expansion","path":"expand/expansion.json","kind":"value"}],
                  "params":{"value":"ignored"},
                  "dynamic":{"expansion_output":"expansion"}
                },
                {
                  "id":"publish",
                  "kind":"shell",
                  "inputs":["report"],
                  "outputs":[{"name":"out","path":"publish/out.txt"}],
                  "effects":["filesystem"],
                  "params":{"argv":["/bin/sh","-c","cat ../inputs/report > ../outputs/out.txt"]}
                }
              ],
              "edges":[]
            }"#,
        )
        .expect("graph")
    }

    #[test]
    fn parses_dynamic_expansion_document() {
        let document = parse_dynamic_expansion_document(
            &serde_json::to_string(&json!({
                "schema_version": DYNAMIC_EXPANSION_SCHEMA_VERSION,
                "nodes": [],
                "edges": []
            }))
            .expect("json"),
        )
        .expect("document");
        assert_eq!(document.schema_version, DYNAMIC_EXPANSION_SCHEMA_VERSION);
    }

    #[test]
    fn rewrites_generated_ids_and_param_refs() {
        let graph = base_graph();
        let document: DynamicExpansionDocument = serde_json::from_value(json!({
            "schema_version": DYNAMIC_EXPANSION_SCHEMA_VERSION,
            "nodes": [
                {
                    "id":"regional_report",
                    "kind":"shell",
                    "inputs":["seed"],
                    "outputs":[{"name":"report","path":"regional/report.txt"}],
                    "effects":["filesystem"],
                    "params":{
                        "region":"north",
                        "upstream":{"node_output":{"node_id":"regional_report","output_name":"report"}}
                    }
                }
            ],
            "edges": [
                {"from":{"node_id":"seed","port":"out"},"to":{"node_id":"regional_report","port":"seed"}},
                {"from":{"node_id":"regional_report","port":"report"},"to":{"node_id":"publish","port":"report"}}
            ]
        }))
        .expect("document");

        let applied = apply_dynamic_expansion(&graph, "expand_regions", document).expect("apply");
        assert!(applied.graph.nodes.iter().any(|node| node.id == "expand_regions__regional_report"));
        let generated = applied
            .graph
            .nodes
            .iter()
            .find(|node| node.id == "expand_regions__regional_report")
            .expect("generated node");
        match &generated.params {
            ParamValue::Object(map) => {
                match map.get("upstream") {
                    Some(ParamValue::Ref(reference)) => {
                        assert_eq!(reference.graph_input, None);
                        assert_eq!(reference.path_var, None);
                        match reference.node_output.as_ref() {
                            Some(NodeOutputRef { node_id, output_name }) => {
                                assert_eq!(node_id, "expand_regions__regional_report");
                                assert_eq!(output_name, "report");
                            }
                            None => panic!("expected node output ref"),
                        }
                    }
                    other => panic!("expected upstream ref, got {other:?}"),
                }
            }
            _ => panic!("expected params object"),
        }
        assert_eq!(
            applied.record,
            DynamicExpansionRecord {
                controller_node_id: "expand_regions".to_string(),
                expansion_output: "expansion".to_string(),
                expansion_fingerprint: applied.record.expansion_fingerprint.clone(),
                generated_node_ids: vec!["expand_regions__regional_report".to_string()],
                generated_edge_count: 2,
            }
        );
    }

    #[test]
    fn rejects_controller_edges_in_base_graph() {
        let mut graph = base_graph();
        graph.edges.push(Edge {
            id: None,
            kind: crate::EdgeKind::Data,
            decision: None,
            from: crate::PortRef {
                node_id: "expand_regions".to_string(),
                port: "expansion".to_string(),
            },
            to: crate::PortRef {
                node_id: "publish".to_string(),
                port: "report".to_string(),
            },
        });
        let document = DynamicExpansionDocument {
            schema_version: DYNAMIC_EXPANSION_SCHEMA_VERSION.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        let error = apply_dynamic_expansion(&graph, "expand_regions", document).expect_err("error");
        assert!(error.contains("must not have declared graph edges"));
    }

    #[test]
    fn helper_namespaces_generated_ids() {
        assert_eq!(
            generated_node_id("expand_regions", "report"),
            "expand_regions__report"
        );
    }
}
