#![forbid(unsafe_code)]
//! Public telemetry facade for runtime crates and external callers.

pub use crate::shared::telemetry::{
    exit_code_kind, truncate_chars, TelemetrySpan, MAX_COMMAND_FIELD_CHARS, MAX_TEXT_FIELD_CHARS,
    TELEMETRY_FILE_ENV, TELEMETRY_INCLUDE_ARGS_ENV,
};
