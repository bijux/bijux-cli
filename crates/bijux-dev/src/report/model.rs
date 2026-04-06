use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandReport {
    pub command_id: String,
    pub status: String,
    pub effect: String,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub details: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::CommandReport;

    #[test]
    fn command_report_roundtrip_is_stable() {
        let report = CommandReport {
            command_id: "verify.evidence-battle".to_string(),
            status: "ok".to_string(),
            effect: "validation".to_string(),
            started_unix_ms: 10,
            finished_unix_ms: 20,
            details: serde_json::json!({"count": 3}),
        };
        let payload = serde_json::to_string_pretty(&report).expect("serialize");
        let parsed: CommandReport = serde_json::from_str(&payload).expect("deserialize");
        assert_eq!(parsed, report);
    }
}
