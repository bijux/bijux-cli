//! Status contract inventory and execution services.

mod id;
mod kind;
mod registry;
mod runner;
mod spec;

pub use registry::{build_inventory_report, status_contract_specs};
pub use runner::{run_all_contracts, run_contract};
