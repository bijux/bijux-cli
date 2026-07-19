//! Reusable subgraph expansion into plain DAG nodes and edges.

use crate::canonical::is_valid_canonical_name;
use crate::{
    materialize_graph_input_value, Graph, GraphInputSpec, NodeOutputRef, ParamValue, RefSpec,
    Severity, SubgraphDefinition, SubgraphInstance, ValidationDiagnostic,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphExpansionError {
    pub code: &'static str,
    pub message: String,
    pub path: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExpandedGraph {
    pub graph: Graph,
    pub instance_exports: BTreeMap<String, BTreeMap<String, NodeOutputRef>>,
}

#[derive(Debug, Clone)]
struct ExpandedSubgraphTemplate {
    inputs: BTreeMap<String, GraphInputSpec>,
    graph: Graph,
    outputs: BTreeMap<String, NodeOutputRef>,
}

pub fn expand_graph(graph: &Graph) -> Result<Graph, GraphExpansionError> {
    Ok(expand_graph_with_exports(graph)?.graph)
}

pub fn expand_graph_with_exports(graph: &Graph) -> Result<ExpandedGraph, GraphExpansionError> {
    expand_graph_inner(graph, "/")
}

pub fn expansion_error_diagnostic(error: GraphExpansionError) -> ValidationDiagnostic {
    ValidationDiagnostic {
        code: error.code.to_string(),
        message: error.message,
        path: error.path,
        hint: error.hint,
        severity: Severity::Error,
    }
}

fn expand_graph_inner(graph: &Graph, path: &str) -> Result<ExpandedGraph, GraphExpansionError> {
    let templates = build_subgraph_templates(graph, path)?;
    let root_node_ids = graph.nodes.iter().map(|node| node.id.as_str()).collect::<BTreeSet<_>>();
    let mut instance_ids = BTreeSet::new();
    let mut expanded_nodes = graph.nodes.clone();
    let mut expanded_edges = graph.edges.clone();
    let mut instance_exports = BTreeMap::<String, BTreeMap<String, NodeOutputRef>>::new();

    for instance in &graph.subgraph_instances {
        validate_instance_identity(instance, path, &root_node_ids, &instance_ids)?;
        instance_ids.insert(instance.id.clone());

        let template = templates.get(&instance.subgraph).ok_or_else(|| GraphExpansionError {
            code: "E1036",
            message: format!("unknown subgraph definition: {}", instance.subgraph),
            path: format!("{path}subgraph_instances/{}", instance.id),
            hint: Some("Define the subgraph before instantiating it".to_string()),
        })?;

        let scoped = scope_subgraph_template(template, instance, path)?;
        let scoped_exports = scoped
            .instance_exports
            .get(&instance.id)
            .cloned()
            .expect("scoped subgraph exports must be recorded under the instance id");
        instance_exports.insert(instance.id.clone(), scoped_exports);
        expanded_nodes.extend(scoped.graph.nodes);
        expanded_edges.extend(scoped.graph.edges);
    }

    let mut expanded_graph = Graph {
        spec: graph.spec.clone(),
        meta: graph.meta.clone(),
        inputs: graph.inputs.clone(),
        nondeterminism_allowed: graph.nondeterminism_allowed,
        subgraphs: BTreeMap::new(),
        subgraph_instances: Vec::new(),
        nodes: expanded_nodes,
        edges: expanded_edges,
    };

    rewrite_instance_output_aliases(&mut expanded_graph, &instance_exports, path)?;

    Ok(ExpandedGraph { graph: expanded_graph, instance_exports })
}

fn build_subgraph_templates(
    graph: &Graph,
    path: &str,
) -> Result<BTreeMap<String, ExpandedSubgraphTemplate>, GraphExpansionError> {
    let mut cache = BTreeMap::new();
    let mut resolving = BTreeSet::new();
    for definition_name in graph.subgraphs.keys() {
        resolve_subgraph_template(definition_name, graph, path, &mut cache, &mut resolving)?;
    }
    Ok(cache)
}

fn resolve_subgraph_template(
    definition_name: &str,
    graph: &Graph,
    path: &str,
    cache: &mut BTreeMap<String, ExpandedSubgraphTemplate>,
    resolving: &mut BTreeSet<String>,
) -> Result<(), GraphExpansionError> {
    if cache.contains_key(definition_name) {
        return Ok(());
    }
    if !resolving.insert(definition_name.to_string()) {
        return Err(GraphExpansionError {
            code: "E1036",
            message: format!("cyclic subgraph definition: {definition_name}"),
            path: format!("{path}subgraphs/{definition_name}"),
            hint: Some("Break recursive reusable subgraph references".to_string()),
        });
    }

    let definition = graph.subgraphs.get(definition_name).ok_or_else(|| GraphExpansionError {
        code: "E1036",
        message: format!("missing subgraph definition: {definition_name}"),
        path: format!("{path}subgraphs/{definition_name}"),
        hint: None,
    })?;

    let definition_path = format!("{path}subgraphs/{definition_name}/");
    let expanded = expand_graph_inner(&definition.graph, &format!("{definition_path}graph/"))?;
    let outputs = resolve_subgraph_outputs(
        definition,
        &expanded.graph,
        &expanded.instance_exports,
        &format!("{definition_path}outputs/"),
    )?;

    cache.insert(
        definition_name.to_string(),
        ExpandedSubgraphTemplate {
            inputs: expanded.graph.inputs.clone(),
            graph: expanded.graph,
            outputs,
        },
    );
    resolving.remove(definition_name);
    Ok(())
}

fn resolve_subgraph_outputs(
    definition: &SubgraphDefinition,
    expanded_graph: &Graph,
    nested_exports: &BTreeMap<String, BTreeMap<String, NodeOutputRef>>,
    path: &str,
) -> Result<BTreeMap<String, NodeOutputRef>, GraphExpansionError> {
    let mut resolved = BTreeMap::new();
    for (export_name, reference) in &definition.outputs {
        if !is_valid_canonical_name(export_name) {
            return Err(GraphExpansionError {
                code: "E1037",
                message: format!("illegal subgraph output name: {export_name}"),
                path: format!("{path}{export_name}"),
                hint: Some("Use [a-zA-Z0-9_-] only".to_string()),
            });
        }
        let rewritten = rewrite_export_reference(reference, nested_exports, path)?;
        if !expanded_graph.nodes.iter().find(|node| node.id == rewritten.node_id).is_some_and(
            |node| node.outputs.iter().any(|output| output.name == rewritten.output_name),
        ) {
            return Err(GraphExpansionError {
                code: "E1037",
                message: format!(
                    "subgraph output {} points at unknown node output {}.{}",
                    export_name, rewritten.node_id, rewritten.output_name
                ),
                path: format!("{path}{export_name}"),
                hint: Some("Export a declared output from a node inside the subgraph".to_string()),
            });
        }
        resolved.insert(export_name.clone(), rewritten);
    }
    Ok(resolved)
}

fn validate_instance_identity(
    instance: &SubgraphInstance,
    path: &str,
    root_node_ids: &BTreeSet<&str>,
    instance_ids: &BTreeSet<String>,
) -> Result<(), GraphExpansionError> {
    if !is_valid_canonical_name(&instance.id) {
        return Err(GraphExpansionError {
            code: "E1036",
            message: format!("illegal subgraph instance id: {}", instance.id),
            path: format!("{path}subgraph_instances/{}", instance.id),
            hint: Some("Use [a-zA-Z0-9_-] only".to_string()),
        });
    }
    if root_node_ids.contains(instance.id.as_str()) {
        return Err(GraphExpansionError {
            code: "E1036",
            message: format!("subgraph instance id collides with node id: {}", instance.id),
            path: format!("{path}subgraph_instances/{}", instance.id),
            hint: Some("Choose an instance id that does not match a root node id".to_string()),
        });
    }
    if instance_ids.contains(&instance.id) {
        return Err(GraphExpansionError {
            code: "E1036",
            message: format!("duplicate subgraph instance id: {}", instance.id),
            path: format!("{path}subgraph_instances/{}", instance.id),
            hint: Some("Use a unique subgraph instance id".to_string()),
        });
    }
    Ok(())
}

fn scope_subgraph_template(
    template: &ExpandedSubgraphTemplate,
    instance: &SubgraphInstance,
    path: &str,
) -> Result<ExpandedGraph, GraphExpansionError> {
    let input_bindings = resolve_instance_bindings(template, instance, path)?;
    let mut scoped_nodes = Vec::with_capacity(template.graph.nodes.len());
    for node in &template.graph.nodes {
        let mut scoped_node = node.clone();
        scoped_node.id = scoped_node_id(&instance.id, &node.id);
        if let Some(group) = &node.group {
            scoped_node.group = Some(scoped_node_id(&instance.id, group));
        }
        for output in &mut scoped_node.outputs {
            output.path = scoped_output_path(&instance.id, &output.path);
        }
        rewrite_scoped_param_value(
            &mut scoped_node.params,
            &instance.id,
            &input_bindings,
            &instance.id,
            path,
        )?;
        scoped_nodes.push(scoped_node);
    }

    let mut scoped_edges = Vec::with_capacity(template.graph.edges.len());
    for edge in &template.graph.edges {
        let mut scoped_edge = edge.clone();
        scoped_edge.from.node_id = scoped_node_id(&instance.id, &edge.from.node_id);
        scoped_edge.to.node_id = scoped_node_id(&instance.id, &edge.to.node_id);
        if let Some(edge_id) = &edge.id {
            scoped_edge.id = Some(scoped_node_id(&instance.id, edge_id));
        }
        scoped_edges.push(scoped_edge);
    }

    let scoped_outputs = template
        .outputs
        .iter()
        .map(|(export_name, reference)| {
            (
                export_name.clone(),
                NodeOutputRef {
                    node_id: scoped_node_id(&instance.id, &reference.node_id),
                    output_name: reference.output_name.clone(),
                },
            )
        })
        .collect();

    Ok(ExpandedGraph {
        graph: Graph {
            spec: template.graph.spec.clone(),
            meta: template.graph.meta.clone(),
            inputs: BTreeMap::new(),
            nondeterminism_allowed: template.graph.nondeterminism_allowed,
            subgraphs: BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: scoped_nodes,
            edges: scoped_edges,
        },
        instance_exports: BTreeMap::from([(instance.id.clone(), scoped_outputs)]),
    })
}

fn resolve_instance_bindings(
    template: &ExpandedSubgraphTemplate,
    instance: &SubgraphInstance,
    path: &str,
) -> Result<BTreeMap<String, ParamValue>, GraphExpansionError> {
    for input_name in instance.input_bindings.keys() {
        if !template.inputs.contains_key(input_name) {
            return Err(GraphExpansionError {
                code: "E1037",
                message: format!("unknown subgraph input binding: {input_name}"),
                path: format!(
                    "{path}subgraph_instances/{}/input_bindings/{input_name}",
                    instance.id
                ),
                hint: Some("Bind only declared subgraph inputs".to_string()),
            });
        }
    }

    let mut resolved = BTreeMap::new();
    for (input_name, spec) in &template.inputs {
        if let Some(binding) = instance.input_bindings.get(input_name) {
            resolved.insert(input_name.clone(), binding.clone());
            continue;
        }
        if let Some(value) = spec.effective_value() {
            let materialized = materialize_graph_input_value(
                spec,
                value,
                &format!("{path}subgraph_instances/{}/input_bindings/{input_name}", instance.id),
            )
            .map_err(|error| GraphExpansionError {
                code: "E1037",
                message: error.message,
                path: format!(
                    "{path}subgraph_instances/{}/input_bindings/{input_name}",
                    instance.id
                ),
                hint: Some("Provide a valid default or explicit binding".to_string()),
            })?;
            resolved.insert(input_name.clone(), ParamValue::Literal(materialized));
            continue;
        }
        return Err(GraphExpansionError {
            code: "E1037",
            message: format!("missing required subgraph input binding: {input_name}"),
            path: format!("{path}subgraph_instances/{}/input_bindings/{input_name}", instance.id),
            hint: Some("Bind every required subgraph input".to_string()),
        });
    }

    Ok(resolved)
}

fn rewrite_scoped_param_value(
    value: &mut ParamValue,
    scope_id: &str,
    input_bindings: &BTreeMap<String, ParamValue>,
    instance_id: &str,
    path: &str,
) -> Result<(), GraphExpansionError> {
    match value {
        ParamValue::Array(items) => {
            for item in items {
                rewrite_scoped_param_value(item, scope_id, input_bindings, instance_id, path)?;
            }
        }
        ParamValue::Object(map) => {
            for nested in map.values_mut() {
                rewrite_scoped_param_value(nested, scope_id, input_bindings, instance_id, path)?;
            }
        }
        ParamValue::Ref(reference) => {
            if let Some(input_name) = &reference.graph_input {
                *value =
                    input_bindings.get(input_name).cloned().ok_or_else(|| GraphExpansionError {
                        code: "E1037",
                        message: format!("missing subgraph input binding: {input_name}"),
                        path: format!(
                            "{path}subgraph_instances/{instance_id}/input_bindings/{input_name}"
                        ),
                        hint: Some("Bind every required subgraph input".to_string()),
                    })?;
                return Ok(());
            }
            if let Some(node_output) = &mut reference.node_output {
                node_output.node_id = scoped_node_id(scope_id, &node_output.node_id);
            }
        }
        ParamValue::Literal(_) => {}
    }
    Ok(())
}

fn rewrite_instance_output_aliases(
    graph: &mut Graph,
    instance_exports: &BTreeMap<String, BTreeMap<String, NodeOutputRef>>,
    path: &str,
) -> Result<(), GraphExpansionError> {
    for node in &mut graph.nodes {
        rewrite_export_aliases_in_param_value(&mut node.params, instance_exports, path)?;
    }
    for edge in &mut graph.edges {
        if let Some(exports) = instance_exports.get(&edge.from.node_id) {
            let target = exports.get(&edge.from.port).ok_or_else(|| GraphExpansionError {
                code: "E1037",
                message: format!(
                    "subgraph instance output {}.{} is not exposed",
                    edge.from.node_id, edge.from.port
                ),
                path: format!("{path}edges/from/{}/{}", edge.from.node_id, edge.from.port),
                hint: Some("Reference one of the declared subgraph outputs".to_string()),
            })?;
            edge.from.node_id = target.node_id.clone();
            edge.from.port = target.output_name.clone();
        }
        if instance_exports.contains_key(&edge.to.node_id) {
            return Err(GraphExpansionError {
                code: "E1038",
                message: format!(
                    "edges cannot target subgraph instance inputs: {}.{}",
                    edge.to.node_id, edge.to.port
                ),
                path: format!("{path}edges/to/{}/{}", edge.to.node_id, edge.to.port),
                hint: Some(
                    "Bind reusable subgraph inputs with subgraph_instances[].input_bindings"
                        .to_string(),
                ),
            });
        }
    }
    Ok(())
}

fn rewrite_export_aliases_in_param_value(
    value: &mut ParamValue,
    instance_exports: &BTreeMap<String, BTreeMap<String, NodeOutputRef>>,
    path: &str,
) -> Result<(), GraphExpansionError> {
    match value {
        ParamValue::Array(items) => {
            for item in items {
                rewrite_export_aliases_in_param_value(item, instance_exports, path)?;
            }
        }
        ParamValue::Object(map) => {
            for nested in map.values_mut() {
                rewrite_export_aliases_in_param_value(nested, instance_exports, path)?;
            }
        }
        ParamValue::Ref(RefSpec { node_output: Some(reference), .. }) => {
            *reference = rewrite_export_reference(reference, instance_exports, path)?;
        }
        ParamValue::Ref(_) | ParamValue::Literal(_) => {}
    }
    Ok(())
}

fn rewrite_export_reference(
    reference: &NodeOutputRef,
    instance_exports: &BTreeMap<String, BTreeMap<String, NodeOutputRef>>,
    path: &str,
) -> Result<NodeOutputRef, GraphExpansionError> {
    let Some(exports) = instance_exports.get(&reference.node_id) else {
        return Ok(reference.clone());
    };
    exports.get(&reference.output_name).cloned().ok_or_else(|| GraphExpansionError {
        code: "E1037",
        message: format!(
            "subgraph instance output {}.{} is not exposed",
            reference.node_id, reference.output_name
        ),
        path: path.to_string(),
        hint: Some("Reference one of the declared subgraph outputs".to_string()),
    })
}

fn scoped_node_id(instance_id: &str, local_id: &str) -> String {
    format!("{instance_id}__{local_id}")
}

fn scoped_output_path(instance_id: &str, output_path: &str) -> String {
    format!("subgraphs/{instance_id}/{output_path}")
}
