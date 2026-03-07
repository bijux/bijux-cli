use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExportCommandResponse {
    pub output: String,
}
