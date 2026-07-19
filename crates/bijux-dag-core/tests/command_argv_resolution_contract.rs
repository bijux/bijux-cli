use bijux_dag_core::parse_graph_strict;

#[test]
fn resolve_graph_resolves_shell_argv_expressions_with_params_inputs_outputs_and_scalars() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "inputs":{"threads":8},
          "nodes":[
            {
              "id":"seed",
              "kind":"const",
              "outputs":[{"name":"out","path":"seed/out.txt"}],
              "params":{"value":"seed"}
            },
            {
              "id":"align",
              "kind":"shell",
              "inputs":["reads"],
              "outputs":[{"name":"bam","path":"align/out.bam"}],
              "params":{
                "threads":{"graph_input":"threads"},
                "argv":[
                  "aligner",
                  "--threads={params.threads}",
                  "{inputs.reads}",
                  "{outputs.bam}",
                  {"graph_input":"threads"}
                ]
              }
            }
          ],
          "edges":[
            {"from":{"node_id":"seed","port":"out"},"to":{"node_id":"align","port":"reads"}}
          ]
        }"#,
    )
    .expect("parse graph");

    let resolved = graph.resolve_graph().expect("resolve graph");
    let argv = resolved.resolved_params["align"]["argv"]
        .as_array()
        .expect("argv array")
        .iter()
        .map(|entry| entry.as_str().expect("argv string").to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        argv,
        vec![
            "aligner".to_string(),
            "--threads=8".to_string(),
            "{inputs_dir}/seed/reads".to_string(),
            "{outputs_dir}/align/out.bam".to_string(),
            "8".to_string(),
        ]
    );
}

#[test]
fn resolve_graph_rejects_missing_command_argv_params() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"run",
              "kind":"shell",
              "outputs":[{"name":"out","path":"out.txt"}],
              "params":{"argv":["echo","{params.missing}"]}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("parse graph");

    assert!(graph.resolve_graph().is_err(), "missing command template must fail");
}

#[test]
fn container_fingerprint_tracks_resolved_command_argv() {
    let graph_a = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"pack",
              "kind":"container",
              "outputs":[{"name":"bundle","path":"bundle.tar"}],
              "params":{"threads":2},
              "container":{
                "image":"alpine:3.20",
                "argv":["tool","--threads={params.threads}","{outputs.bundle}"],
                "engine":"docker"
              }
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("parse graph a");
    let graph_b = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"pack",
              "kind":"container",
              "outputs":[{"name":"bundle","path":"bundle.tar"}],
              "params":{"threads":4},
              "container":{
                "image":"alpine:3.20",
                "argv":["tool","--threads={params.threads}","{outputs.bundle}"],
                "engine":"docker"
              }
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("parse graph b");

    let node_a = &graph_a.nodes[0];
    let node_b = &graph_b.nodes[0];
    let resolved_a = graph_a.resolve_graph().expect("resolve a");
    let resolved_b = graph_b.resolve_graph().expect("resolve b");

    let fp_a = graph_a
        .node_fingerprint_with_params(node_a, &resolved_a.resolved_params["pack"])
        .expect("fingerprint a");
    let fp_b = graph_b
        .node_fingerprint_with_params(node_b, &resolved_b.resolved_params["pack"])
        .expect("fingerprint b");

    assert_ne!(fp_a, fp_b, "container argv resolution must affect fingerprint");
}
