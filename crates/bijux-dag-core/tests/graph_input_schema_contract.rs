use bijux_dag_core::parse_graph_strict;
use serde_json::json;

#[test]
fn typed_graph_inputs_cover_supported_schema_shapes() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"typed-inputs","owners":[],"tags":[]},
          "inputs":{
            "region":{"type":"string","default":"eu-west-1"},
            "attempts":{"type":"integer","default":3},
            "threshold":{"type":"float","default":0.75},
            "dry_run":{"type":"boolean","default":false},
            "workspace_path":{"type":"path","default":"workspace/data"},
            "mode":{"type":"enum","values":["daily","adhoc"],"default":"daily"},
            "samples":{"type":"array","items":{"type":"string"},"default":["alpha","beta"]},
            "payload":{
              "type":"object",
              "properties":{
                "tenant":{"type":"string","required":true},
                "labels":{"type":"array","items":{"type":"string"},"default":["science"]},
                "enabled":{"type":"boolean","default":true}
              },
              "default":{"tenant":"atlas"}
            }
          },
          "nodes":[
            {
              "id":"emit",
              "kind":"const",
              "inputs":[],
              "outputs":[{"name":"value","path":"emit/value.json"}],
              "params":{
                "region":{"graph_input":"region"},
                "payload":{"graph_input":"payload"}
              }
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");

    let diagnostics = graph.validate_with_warnings();
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.severity != bijux_dag_core::Severity::Error));

    let effective = graph.effective_inputs().expect("effective inputs");
    assert_eq!(effective["region"], "eu-west-1");
    assert_eq!(effective["attempts"], 3);
    assert_eq!(effective["threshold"], json!(0.75));
    assert_eq!(effective["dry_run"], false);
    assert_eq!(effective["workspace_path"], "workspace/data");
    assert_eq!(effective["mode"], "daily");
    assert_eq!(effective["samples"], json!(["alpha", "beta"]));
    assert_eq!(effective["payload"]["tenant"], "atlas");
    assert_eq!(effective["payload"]["labels"], json!(["science"]));
    assert_eq!(effective["payload"]["enabled"], true);

    let schema = graph.input_schema();
    assert_eq!(schema["mode"]["type"], "enum");
    assert_eq!(schema["samples"]["items"]["type"], "string");
    assert_eq!(schema["payload"]["properties"]["tenant"]["required"], true);
}

#[test]
fn invalid_typed_graph_input_default_reports_exact_path() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"typed-input-errors","owners":[],"tags":[]},
          "inputs":{
            "payload":{
              "type":"object",
              "properties":{
                "tenant":{"type":"string","required":true}
              },
              "default":{"tenant":7}
            }
          },
          "nodes":[
            {
              "id":"emit",
              "kind":"const",
              "inputs":[],
              "outputs":[{"name":"value","path":"emit/value.json"}],
              "params":{"value":{"graph_input":"payload"}}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");

    let diagnostics = graph.validate_with_warnings();
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "E1033")
        .expect("typed input default diagnostic");
    assert_eq!(diagnostic.path, "/inputs/payload/default/tenant");
    assert!(diagnostic.message.contains("expected string"));
}
