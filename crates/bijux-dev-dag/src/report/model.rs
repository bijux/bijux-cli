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
