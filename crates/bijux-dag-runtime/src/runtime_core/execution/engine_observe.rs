use serde_json::json;

pub fn node_eligible_events(node_ids: &[String], ts: u128) -> Vec<serde_json::Value> {
    node_ids
        .iter()
        .map(|node_id| {
            json!({
                "event": "node_eligible",
                "ts": ts,
                "node_id": node_id,
            })
        })
        .collect()
}
