#![forbid(unsafe_code)]
//! Read-only contract schema inventory interfaces for maintainer tooling.

/// Structured schema inventory queried from durable contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractsSchemaQuery {
    /// Stable schema ids exposed by contract types.
    pub schema_ids: Vec<String>,
    /// Schema inventory version marker.
    pub schema_version: String,
}

/// Query contracts/schema inventory without presentation formatting.
#[must_use]
pub fn contracts_schema_query() -> ContractsSchemaQuery {
    ContractsSchemaQuery {
        schema_ids: vec![
            "output-envelope-v1".to_string(),
            "error-envelope-v1".to_string(),
            "plugin-manifest-v2".to_string(),
            "product-mount-descriptor-v1".to_string(),
        ],
        schema_version: "v2".to_string(),
    }
}
