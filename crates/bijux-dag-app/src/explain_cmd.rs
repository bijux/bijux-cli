use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExplainCommandResponse {
    pub status: String,
}
