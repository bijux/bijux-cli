//! Native status contract inventory and execution logic.

mod catalog;
mod control_plane;
mod quality;
mod resilience;
mod run;
mod runtime;

pub(crate) use catalog::native_status_contract_rows;
pub(crate) use run::run_native_status_contract;
