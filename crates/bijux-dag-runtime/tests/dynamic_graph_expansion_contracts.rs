use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{Runtime, RuntimeConfig};
use serde_json::Value;
use std::fs;

#[test]
fn runtime_executes_dynamic_expansion_before_planning() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"expand",
              "kind":"const",
              "semantic_kind":"dynamic",
              "outputs":[{"name":"expansion","path":"expand/expansion.json","kind":"value"}],
              "params":{
                "value":{
                  "schema_version":"bijux-dag-dynamic-expansion/v0.1",
                  "nodes":[
                    {
                      "id":"regional_report",
                      "kind":"const",
                      "outputs":[{"name":"report","path":"regional/report.json","kind":"value"}],
                      "params":{"value":"north"}
                    },
                    {
                      "id":"publish",
                      "kind":"shell",
                      "inputs":["report"],
                      "outputs":[{"name":"out","path":"out.txt"}],
                      "effects":["filesystem"],
                      "params":{
                        "argv":[
                          "/bin/sh",
                          "-c",
                          "cat ../inputs/expand__regional_report/report > ../outputs/out.txt"
                        ]
                      }
                    }
                  ],
                  "edges":[
                    {
                      "from":{"node_id":"regional_report","port":"report"},
                      "to":{"node_id":"publish","port":"report"}
                    }
                  ]
                }
              },
              "dynamic":{"expansion_output":"expansion"}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");

    let dir = tempfile::tempdir().expect("tempdir");
    let final_path =
        Runtime::new().run(&graph, dir.path(), RuntimeConfig::default()).expect("dynamic run");

    let published = fs::read_to_string(
        final_path.join("nodes").join("expand__publish").join("outputs").join("out.txt"),
    )
    .expect("published output");
    assert!(published.contains("north"));

    let snapshot: Value = serde_json::from_str(
        &fs::read_to_string(final_path.join("graph.snapshot.json")).expect("graph snapshot"),
    )
    .expect("snapshot json");
    let node_ids = snapshot["graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .filter_map(|node| node.get("id").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert_eq!(node_ids, vec!["expand__publish", "expand__regional_report"]);
    assert_eq!(
        snapshot["dynamic_expansions"][0]["controller_node_id"],
        Value::String("expand".to_string())
    );
    assert_eq!(snapshot["source_graph"]["nodes"][0]["id"], Value::String("expand".to_string()));
    assert_eq!(
        snapshot["source_graph_fingerprint"],
        Value::String(graph.graph_fingerprint().expect("source fingerprint"))
    );
}
