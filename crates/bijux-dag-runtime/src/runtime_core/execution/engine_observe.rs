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

#[cfg(test)]
mod tests {
    use super::node_eligible_events;

    #[test]
    fn emits_one_event_per_node_with_stable_payload() {
        let node_ids = vec!["alpha".to_string(), "beta".to_string()];
        let events = node_eligible_events(&node_ids, 42);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["event"], "node_eligible");
        assert_eq!(events[0]["ts"], 42);
        assert_eq!(events[0]["node_id"], "alpha");
        assert_eq!(events[1]["event"], "node_eligible");
        assert_eq!(events[1]["ts"], 42);
        assert_eq!(events[1]["node_id"], "beta");
    }
}
