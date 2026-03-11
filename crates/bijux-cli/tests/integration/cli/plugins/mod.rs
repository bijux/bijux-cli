#![forbid(unsafe_code)]
//! plugins integration suites.

mod plugin_cli_lifecycle;
mod plugin_command_parity;
mod plugin_discovery_determinism_matrix;
mod plugin_failure_injection;
mod plugin_failure_rollback_matrix;
mod plugin_lifecycle_matrix;
mod plugin_namespace_law;
mod plugin_scaffold_fuzz_regressions;
mod plugin_scaffold_fuzz_targets;
mod plugin_scaffold_minimal;
