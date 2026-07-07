use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{node_io_contract, parse_graph_strict, ParamBindingSource};

#[test]
fn tutorial_examples_parse_as_stable_contracts() {
    let examples = [
        "../../evidence/authoring/examples/hello.dag.json",
        "../../evidence/authoring/examples/etl-constant-to-shell.dag.json",
        "../../evidence/authoring/examples/cached-branched-report.dag.json",
        "../../evidence/authoring/examples/file-processing-report.dag.json",
        "../../evidence/authoring/examples/release-note-bundle.dag.json",
        "../../evidence/authoring/examples/regional-sales-pipeline.dag.json",
        "../../evidence/authoring/examples/multi-output-artifact.dag.json",
        "../../evidence/authoring/examples/replay-heavy-branching.dag.json",
        "../../evidence/authoring/examples/failure-heavy-retry.dag.json",
        "../../evidence/authoring/examples/parameterized-report.dag.json",
    ];
    for relative in examples {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let _graph = parse_graph_strict(&text)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
    }
}

#[test]
fn parameterized_report_example_uses_graph_inputs_and_output_references() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/authoring/examples/parameterized-report.dag.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let graph = parse_graph_strict(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));

    let effective_inputs = graph.effective_inputs().expect("effective inputs");
    assert_eq!(effective_inputs.get("region"), Some(&serde_json::json!("eu-west-1")));
    assert_eq!(graph.input_schema()["publish_channel"]["type"], "enum");

    let publish_contract =
        node_io_contract(&graph, "publish_summary").expect("publish_summary contract");
    assert!(publish_contract.param_bindings.iter().any(|binding| matches!(
        &binding.source,
        ParamBindingSource::GraphInput { input_name } if input_name == "publish_channel"
    )));
    assert!(publish_contract.param_bindings.iter().any(|binding| matches!(
        &binding.source,
        ParamBindingSource::NodeOutput { node_id, output_name }
            if node_id == "build_report" && output_name == "report"
    )));

    let publish_node =
        graph.nodes.iter().find(|node| node.id == "publish_summary").expect("publish_summary node");
    assert!(!publish_node.cache.enabled);
    assert_eq!(publish_node.cache.reason.as_deref(), Some("publishes externally visible summary"));
    assert_eq!(publish_node.env_allowlist, vec!["REPORT_CHANNEL".to_string()]);
}

#[test]
fn multi_output_example_declares_typed_and_optional_outputs() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/authoring/examples/multi-output-artifact.dag.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let graph = parse_graph_strict(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));

    let contract = node_io_contract(&graph, "produce_outputs").expect("produce_outputs contract");
    assert_eq!(contract.outputs.len(), 2);
    assert_eq!(contract.outputs[0].kind, bijux_dag_core::OutputKind::Value);
    assert_eq!(contract.outputs[0].media_type, "application/json");
    assert_eq!(contract.outputs[1].kind, bijux_dag_core::OutputKind::Log);
    assert!(!contract.outputs[1].required);
    assert_eq!(contract.outputs[1].media_type, "text/plain");
}

#[test]
fn file_processing_report_example_uses_required_path_inputs_and_promotable_report_output() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/authoring/examples/file-processing-report.dag.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let graph = parse_graph_strict(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));

    let source_dir = &graph.input_schema()["source_dir"];
    assert_eq!(source_dir["type"], "path");
    assert_eq!(source_dir["required"], true);

    let report_title = &graph.input_schema()["report_title"];
    assert_eq!(report_title["type"], "string");
    assert_eq!(report_title["default"], "Repository File Processing Report");

    let contract = node_io_contract(&graph, "render_report").expect("render_report contract");
    assert_eq!(contract.outputs.len(), 1);
    assert_eq!(contract.outputs[0].media_type, "text/markdown");
    assert!(contract.outputs[0].promotable);
    assert!(contract.param_bindings.iter().any(|binding| matches!(
        &binding.source,
        ParamBindingSource::GraphInput { input_name } if input_name == "report_title"
    )));
}

#[test]
fn regional_sales_pipeline_example_uses_path_inputs_and_promotable_final_table() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/authoring/examples/regional-sales-pipeline.dag.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let graph = parse_graph_strict(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));

    let orders_csv = &graph.input_schema()["orders_csv"];
    assert_eq!(orders_csv["type"], "path");
    assert_eq!(orders_csv["required"], true);

    let targets_json = &graph.input_schema()["targets_json"];
    assert_eq!(targets_json["type"], "path");
    assert_eq!(targets_json["required"], true);

    let report_title = &graph.input_schema()["report_title"];
    assert_eq!(report_title["type"], "string");
    assert_eq!(report_title["default"], "Regional Revenue Attainment");

    let contract =
        node_io_contract(&graph, "publish_final_table").expect("publish_final_table contract");
    assert_eq!(contract.outputs.len(), 1);
    assert_eq!(contract.outputs[0].media_type, "text/csv");
    assert!(contract.outputs[0].promotable);
    assert!(contract.param_bindings.iter().any(|binding| matches!(
        &binding.source,
        ParamBindingSource::GraphInput { input_name } if input_name == "report_title"
    )));
}

#[test]
fn release_note_bundle_example_uses_path_input_and_pinned_container_image() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/authoring/examples/release-note-bundle.dag.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let graph = parse_graph_strict(&text)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));

    let source_note = &graph.input_schema()["source_note"];
    assert_eq!(source_note["type"], "path");
    assert_eq!(source_note["required"], true);

    let bundle_label = &graph.input_schema()["bundle_label"];
    assert_eq!(bundle_label["type"], "string");
    assert_eq!(bundle_label["default"], "Weekly Release Note");

    let package_node =
        graph.nodes.iter().find(|node| node.id == "package_bundle").expect("package_bundle node");
    let container = package_node.container.as_ref().expect("container spec");
    assert_eq!(container.engine, "docker");
    assert!(container.image.contains("@sha256:"));
    assert_eq!(container.workdir.as_deref(), Some("{work_dir}/scratch"));
    assert_eq!(container.argv.last().map(String::as_str), Some("{params.bundle_label}"));

    let contract = node_io_contract(&graph, "package_bundle").expect("package_bundle contract");
    assert_eq!(contract.outputs.len(), 2);
    assert!(contract.param_bindings.iter().any(|binding| matches!(
        &binding.source,
        ParamBindingSource::GraphInput { input_name } if input_name == "bundle_label"
    )));
    assert!(contract.outputs.iter().any(|output| output.media_type == "text/plain" && output.promotable));
    assert!(contract.outputs.iter().any(|output| output.media_type == "application/json"));
}
