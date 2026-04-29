//! JSON schema generation helpers for durable contract types.

use schemars::schema::RootSchema;
use schemars::schema_for;

use crate::contracts::{
    ErrorEnvelopeV1, OutputEnvelopeV1, PluginManifestV2, ProductRegistryDocument,
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
