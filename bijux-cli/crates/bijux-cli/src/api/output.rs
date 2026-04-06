#![forbid(unsafe_code)]
//! Output rendering facade.

pub use crate::shared::output::{
    emit_error, emit_success, render_value, EmitError, EmitterConfig, OutputStream, RenderedOutput,
};
