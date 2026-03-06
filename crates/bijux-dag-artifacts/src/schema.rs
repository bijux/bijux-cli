use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactSchemaDescriptor {
    pub name: String,
    pub version: String,
    pub media_type: String,
    pub encoding: String,
    pub validation_mode: SchemaValidationMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaValidationMode {
    Strict,
    Warn,
    Skip,
}

pub fn validate_output_schema_descriptor(descriptor: &ArtifactSchemaDescriptor) -> Result<(), String> {
    if descriptor.name.trim().is_empty() {
        return Err("schema name must not be empty".to_string());
    }
    if descriptor.version.trim().is_empty() {
        return Err("schema version must not be empty".to_string());
    }
    if descriptor.media_type.trim().is_empty() {
        return Err("schema media_type must not be empty".to_string());
    }
    if descriptor.encoding.trim().is_empty() {
        return Err("schema encoding must not be empty".to_string());
    }
    Ok(())
}
