use bijux_dag_app::{dag_command, dag_run};
use std::fs;

fn write_graph_fragments() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let foundation = dir.path().join("foundation.json");
    let publication = dir.path().join("publication.json");
    fs::write(
        &foundation,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"foundation","owners":[],"tags":[]},
          "nodes":[{"id":"extract","kind":"const","outputs":[{"name":"report","path":"extract/report.json"}],"params":{"value":"seed"}}],
          "edges":[]
        }"#,
    )
    .expect("write foundation");
    fs::write(
        &publication,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"publication","owners":[],"tags":[]},
          "nodes":[{"id":"publish","kind":"const","inputs":["report"],"outputs":[{"name":"out","path":"publish/out.json"}],"params":{"seed":{"node_output":{"node_id":"extract","output_name":"report"}}}}],
          "edges":[{"from":{"node_id":"extract","port":"report"},"to":{"node_id":"publish","port":"report"}}]
        }"#,
    )
    .expect("write publication");
    (dir, foundation, publication)
}

#[test]
fn canonicalize_supports_composed_graph_fragments() {
    let (_dir, foundation, publication) = write_graph_fragments();
    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "canonicalize",
            foundation.to_string_lossy().as_ref(),
            publication.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn fingerprint_and_hash_graph_accept_composed_fragments() {
    let (_dir, foundation, publication) = write_graph_fragments();

    let fingerprint_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "fingerprint",
            foundation.to_string_lossy().as_ref(),
            publication.to_string_lossy().as_ref(),
        ])
        .expect("parse fingerprint");
    let fingerprint_code = dag_run(&fingerprint_matches).expect("run fingerprint");
    assert_eq!(fingerprint_code, std::process::ExitCode::SUCCESS);

    let hash_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "hash",
            "graph",
            foundation.to_string_lossy().as_ref(),
            publication.to_string_lossy().as_ref(),
        ])
        .expect("parse hash");
    let hash_code = dag_run(&hash_matches).expect("run hash");
    assert_eq!(hash_code, std::process::ExitCode::SUCCESS);
}

#[test]
fn graph_dot_supports_composed_graph_fragments() {
    let (_dir, foundation, publication) = write_graph_fragments();
    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "graph",
            foundation.to_string_lossy().as_ref(),
            publication.to_string_lossy().as_ref(),
        ])
        .expect("parse graph");

    let code = dag_run(&matches).expect("run graph");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}
