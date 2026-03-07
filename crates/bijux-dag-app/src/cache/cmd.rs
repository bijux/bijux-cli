use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CacheCommandResponse {
    pub status: String,
}
