#![forbid(unsafe_code)]
//! Read-only state and parity/status query interfaces for maintainer tooling.

mod parity_status;
mod state_diagnostics;
pub(crate) mod state_paths;

pub use parity_status::{parity_status_query, ParityStatusQuery};
pub use state_diagnostics::{state_diagnostics_query, StateDiagnosticsQuery, StatePathStatus};
