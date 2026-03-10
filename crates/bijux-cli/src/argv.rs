#![forbid(unsafe_code)]
//! Backward-compatible argv helper shim.

pub(crate) use crate::interface::cli::parser::{command_option_value, command_positionals};
