#![forbid(unsafe_code)]
//! Public telemetry facade for runtime crates and external callers.

pub use crate::shared::telemetry::{
    exit_code_kind, TelemetrySpan, TELEMETRY_FILE_ENV, TELEMETRY_INCLUDE_ARGS_ENV,
};
