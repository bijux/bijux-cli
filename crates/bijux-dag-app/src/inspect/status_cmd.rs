use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StatusCommandResponse {
    pub status: String,
}
