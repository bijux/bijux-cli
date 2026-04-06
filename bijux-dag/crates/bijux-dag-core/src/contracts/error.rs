//! Core error surface.

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid spec version: {0}")]
    InvalidSpec(String),
    #[error("validation failed")]
    ValidationFailed,
}
