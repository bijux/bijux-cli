use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ImportCommandResponse {
    pub summary: String,
}
