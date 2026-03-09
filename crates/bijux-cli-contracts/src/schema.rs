//! JSON schema generation helpers for durable contract types.

use schemars::schema::RootSchema;
use schemars::schema_for;

use crate::contracts::OutputEnvelopeV1;

/// Build a JSON Schema for `OutputEnvelopeV1`.
#[must_use]
pub fn output_envelope_v1_schema() -> RootSchema {
    schema_for!(OutputEnvelopeV1)
}
