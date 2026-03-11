//! Native status contract inventory and execution logic.

mod catalog;
mod runner;

pub(crate) use catalog::native_status_contract_rows;
pub(crate) use runner::run_native_status_contract;
