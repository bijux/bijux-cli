//! Native status contract inventory and execution logic.

mod executors;
mod specs;

pub(crate) use executors::run_native_status_contract;
pub(crate) use specs::native_status_contract_rows;
