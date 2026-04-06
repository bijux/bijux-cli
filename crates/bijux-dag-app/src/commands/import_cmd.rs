use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ImportCommandResponse {
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::ImportCommandResponse;

    #[test]
    fn import_command_response_serializes_stably() {
        let response = ImportCommandResponse { summary: "verified".to_string() };
        let value = serde_json::to_value(&response).expect("serialize import response");
        assert_eq!(value["summary"], "verified");
    }
}
