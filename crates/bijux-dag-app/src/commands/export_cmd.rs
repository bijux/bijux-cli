use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExportCommandResponse {
    pub output: String,
}

#[cfg(test)]
mod tests {
    use super::ExportCommandResponse;

    #[test]
    fn export_command_response_serializes_stably() {
        let response = ExportCommandResponse { output: "bundle.json".to_string() };
        let value = serde_json::to_value(&response).expect("serialize export response");
        assert_eq!(value["output"], "bundle.json");
    }
}
