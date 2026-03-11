//! Maintainer contract inventories and execution helpers.

pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::fs;
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::process::Command;

pub(crate) use serde_json::{json, Value};

mod compliance;
mod generator_runner;
mod migration_reports;
mod native;
mod shared;
mod status_registry;

#[cfg(test)]
mod tests;

pub use compliance::{
    build_audit_report, build_diff_report, build_flaky_tests_report, build_migrated_report,
    build_remaining_report, build_requirement_catalog_report,
};
pub use generator_runner::{build_generators_report, run_all_generators, run_generator};
pub use migration_reports::{
    build_e2e_contract_report, build_package_metadata_report, build_pip_audit_report,
    build_provenance_statement_report, build_python_capture_report,
};
pub use status_registry::{
    build_status_contracts_report, build_status_scripts_report, run_all_status_contracts,
    run_all_status_scripts, run_status_contract, run_status_script,
};

pub(crate) use shared::{
    collect_files, generated_at_utc, rel, run_bijux_json, run_bijux_json_env, run_bijux_text,
    status_slug_for_name, write_json, write_status_artifact_json,
};
