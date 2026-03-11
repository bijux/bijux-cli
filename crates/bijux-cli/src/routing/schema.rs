//! JSON schema generation helpers for durable contract types.

use schemars::schema::RootSchema;
use schemars::schema_for;

use crate::contracts::{ErrorEnvelopeV1, OutputEnvelopeV1, PluginManifestV1};

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

/// Build a JSON Schema for `PluginManifestV1`.
#[must_use]
pub fn plugin_manifest_v1_schema() -> RootSchema {
    schema_for!(PluginManifestV1)
}
