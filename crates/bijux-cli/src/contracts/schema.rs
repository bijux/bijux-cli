//! JSON schema generation helpers for durable contract types.

use schemars::schema::RootSchema;
use schemars::schema_for;

use crate::contracts::{
    CommandEnvelopeV1, ConfigSchemaRegistryV1, ErrorEnvelopeV1, OutputEnvelopeV1, PluginManifestV2,
    ProductMountDescriptor, ProductRegistryDocument,
};

/// Build a JSON Schema for `OutputEnvelopeV1`.
#[must_use]
pub fn output_envelope_v1_schema() -> RootSchema {
    schema_for!(OutputEnvelopeV1)
}

/// Build a JSON Schema for `ErrorEnvelopeV1`.
#[must_use]
pub fn error_envelope_v1_schema() -> RootSchema {
    schema_for!(ErrorEnvelopeV1)
}

/// Build a JSON Schema for `CommandEnvelopeV1`.
#[must_use]
pub fn command_envelope_v1_schema() -> RootSchema {
    schema_for!(CommandEnvelopeV1)
}

/// Build a JSON Schema for `ConfigSchemaRegistryV1`.
#[must_use]
pub fn config_schema_registry_v1_schema() -> RootSchema {
    schema_for!(ConfigSchemaRegistryV1)
}

/// Build a JSON Schema for `PluginManifestV2`.
#[must_use]
pub fn plugin_manifest_v2_schema() -> RootSchema {
    schema_for!(PluginManifestV2)
}

/// Build a JSON Schema for the official product registry contract.
#[must_use]
pub fn official_product_registry_schema() -> RootSchema {
    schema_for!(ProductRegistryDocument)
}

/// Build a JSON Schema for the product mount descriptor contract.
#[must_use]
pub fn product_mount_descriptor_schema() -> RootSchema {
    schema_for!(ProductMountDescriptor)
}
