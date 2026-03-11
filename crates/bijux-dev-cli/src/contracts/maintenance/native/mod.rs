//! Native status contract inventory and execution logic.

mod catalog;
mod handlers;

pub(crate) use catalog::native_status_contract_rows;
pub(crate) use handlers::run_native_status_contract;
