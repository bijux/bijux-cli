#![forbid(unsafe_code)]
//! root integration suites.

mod app_direct_invocation;
mod bin_core_integration;
mod cli_command_matrix;
mod command_execution;
mod command_family_consistency_extra;
mod cross_command_consistency_matrix;
mod cross_surface_equivalence;
mod cross_surface_state_extra;
mod deterministic_output_matrix;
mod exit_code_law_matrix;
mod flag_normalization_matrix;
mod help_snapshots;
mod help_tree_law_extra;
mod metadata_inspection_matrix;
mod parser_invalid_utf8_argv;
mod precedence_matrix;
mod python_command_port_parity;
mod root_command_matrix;
mod stream_discipline_matrix;
