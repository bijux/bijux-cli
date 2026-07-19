//! Maintainer contract inventories and execution helpers.

pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::fs;
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::process::Command;

pub(crate) use serde_json::{json, Value};

mod compliance;
mod generators;
mod inventory;

#[cfg(test)]
mod tests;

pub use super::status::{
    build_inventory_report as build_status_contracts_report,
    run_all_contracts as run_all_status_contracts, run_contract as run_status_contract,
};
pub use compliance::{
    build_audit_report, build_diff_report, build_flaky_tests_report,
    build_ignored_dag_tests_report, build_migrated_report, build_remaining_report,
    build_requirement_catalog_report,
};
pub use generators::{build_generators_report, run_all_generators, run_generator};
pub use inventory::{
    build_e2e_contract_report, build_package_metadata_report, build_pip_audit_report,
    build_provenance_statement_report,
};

pub(crate) use crate::suites::{native_status_contract_rows, run_native_status_contract};
pub(crate) use inventory::{
    collect_files, generated_at_utc, rel, run_bijux_json, run_bijux_json_env, run_bijux_text,
    status_slug_for_name, write_json, write_status_artifact_json,
};
