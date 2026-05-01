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

/// Structured compatibility lanes for durable schema/versioned surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionCompatibilityLaneQuery {
    /// Compatibility lane inventory format version.
    pub schema_version: String,
    /// Version-compatibility lanes per product surface.
    pub surfaces: Vec<VersionCompatibilitySurface>,
}

/// Version compatibility lane for one surface contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionCompatibilitySurface {
    /// Stable surface identifier.
    pub surface: String,
    /// Current versions expected for normal production usage.
    pub current_versions: Vec<String>,
    /// Explicitly accepted previous versions.
    pub accepted_previous_versions: Vec<String>,
    /// Known refused versions to keep failures deterministic.
    pub refused_versions: Vec<String>,
}

/// Query contracts/schema inventory without presentation formatting.
#[must_use]
pub fn contracts_schema_query() -> ContractsSchemaQuery {
    ContractsSchemaQuery {
        schema_ids: vec![
            "command-envelope-v1".to_string(),
            "output-envelope-v1".to_string(),
            "error-envelope-v1".to_string(),
            "config-schema-registry-v1".to_string(),
            "plugin-manifest-v2".to_string(),
            "product-mount-descriptor-v1".to_string(),
        ],
        schema_version: "v4".to_string(),
    }
}

/// Query version-compatibility lanes without presentation formatting.
#[must_use]
pub fn version_compatibility_lanes_query() -> VersionCompatibilityLaneQuery {
    VersionCompatibilityLaneQuery {
        schema_version: "v1".to_string(),
        surfaces: vec![
            VersionCompatibilitySurface {
                surface: "cli-command-envelope".to_string(),
                current_versions: vec!["command-envelope-v1".to_string()],
                accepted_previous_versions: Vec::new(),
                refused_versions: vec![
                    "command-envelope-v0".to_string(),
                    "command-envelope-v2".to_string(),
                ],
            },
            VersionCompatibilitySurface {
                surface: "cli-output-envelope".to_string(),
                current_versions: vec!["output-envelope-v1".to_string()],
                accepted_previous_versions: Vec::new(),
                refused_versions: vec![
                    "output-envelope-v0".to_string(),
                    "output-envelope-v2".to_string(),
                ],
            },
            VersionCompatibilitySurface {
                surface: "cli-error-envelope".to_string(),
                current_versions: vec!["error-envelope-v1".to_string()],
                accepted_previous_versions: Vec::new(),
                refused_versions: vec![
                    "error-envelope-v0".to_string(),
                    "error-envelope-v2".to_string(),
                ],
            },
            VersionCompatibilitySurface {
                surface: "config-schema-registry".to_string(),
                current_versions: vec!["config-schema-registry-v1".to_string()],
                accepted_previous_versions: Vec::new(),
                refused_versions: vec![
                    "config-schema-registry-v0".to_string(),
                    "config-schema-registry-v2".to_string(),
                ],
            },
            VersionCompatibilitySurface {
                surface: "mount-descriptor".to_string(),
                current_versions: vec!["product-mount-descriptor-v1".to_string()],
                accepted_previous_versions: Vec::new(),
                refused_versions: vec![
                    "product-mount-descriptor-v0".to_string(),
                    "product-mount-descriptor-v2".to_string(),
                ],
            },
            VersionCompatibilitySurface {
                surface: "graph-spec".to_string(),
                current_versions: vec!["bijux-dag/v0.1".to_string()],
                accepted_previous_versions: vec![
                    "v1".to_string(),
                    "v0.1".to_string(),
                    "0.1".to_string(),
                ],
                refused_versions: vec!["v9".to_string(), "bijux-dag/v9".to_string()],
            },
            VersionCompatibilitySurface {
                surface: "run-manifest".to_string(),
                current_versions: vec!["run-manifest/v0.1".to_string()],
                accepted_previous_versions: Vec::new(),
                refused_versions: vec![
                    "run-manifest/v0".to_string(),
                    "run-manifest/v2".to_string(),
                ],
            },
            VersionCompatibilitySurface {
                surface: "artifact-index".to_string(),
                current_versions: vec!["run-dir-schema/v0.1".to_string()],
                accepted_previous_versions: Vec::new(),
                refused_versions: vec![
                    "run-dir-schema/v0".to_string(),
                    "run-dir-schema/v2".to_string(),
                ],
            },
            VersionCompatibilitySurface {
                surface: "replay-bundle".to_string(),
                current_versions: vec![
                    "export-bundle/v0.1".to_string(),
                    "proof-bundle/v0.1".to_string(),
                ],
                accepted_previous_versions: Vec::new(),
                refused_versions: vec![
                    "export-bundle/v0".to_string(),
                    "proof-bundle/v0".to_string(),
                    "export-bundle/v2".to_string(),
                    "proof-bundle/v2".to_string(),
                ],
            },
        ],
    }
}
