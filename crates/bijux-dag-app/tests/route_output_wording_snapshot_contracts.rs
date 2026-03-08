use bijux_dag_app::{format_inspect_human, format_show_human};
use serde_json::json;

#[test]
fn route_level_concise_wording_snapshot_is_stable() {
    let summary = json!({
        "run_id":"run-1",
        "status":"success",
        "graph_fingerprint":"g1",
        "submission_source":"manual",
        "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
        "retry_count":0,
        "cache_hits":0,
        "artifact_count":1,
        "failed_nodes":[]
    });
    let rendered = format_show_human(&summary);
    assert_eq!(
        rendered,
        include_str!("snapshots/route_concise_wording.txt")
    );
}

#[test]
fn route_level_detailed_wording_snapshot_is_stable() {
    let summary = json!({
        "run_id":"run-2",
        "status":"failed",
        "graph_fingerprint":"g2",
        "submission_source":"imported",
        "node_counts":{"success":1,"failed":1,"skipped":0,"cached":0},
        "retry_count":2,
        "cache_hits":1,
        "artifact_count":3,
        "failed_nodes":["stage-b"]
    });
    let rendered = format_inspect_human(&summary);
    assert_eq!(
        rendered,
        include_str!("snapshots/route_detailed_wording.txt")
    );
}
