use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GraphCommandResponse {
    pub output: String,
}
