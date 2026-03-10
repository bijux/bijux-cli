#![forbid(unsafe_code)]
//! Read-only state and parity/status query interfaces for maintainer tooling.

pub(crate) mod cli_command;
mod parity_status;
pub(crate) mod root_command;
mod state_diagnostics;

pub use parity_status::{parity_status_query, ParityStatusQuery};
pub use state_diagnostics::{state_diagnostics_query, StateDiagnosticsQuery, StatePathStatus};
