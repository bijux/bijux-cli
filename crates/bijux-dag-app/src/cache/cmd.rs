use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CacheCommandResponse {
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::CacheCommandResponse;

    #[test]
    fn cache_command_response_serializes_status() {
        let response = CacheCommandResponse { status: "ok".to_string() };
        let value = serde_json::to_value(&response).expect("serialize cache response");
        assert_eq!(value["status"], "ok");
    }
}
