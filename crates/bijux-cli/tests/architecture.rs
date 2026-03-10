#![forbid(unsafe_code)]
//! Architecture and ownership boundary suites for bijux-cli.

#[path = "architecture/architecture_boundaries.rs"]
mod architecture_boundaries;

#[path = "architecture/cli_kernel_domain_boundaries.rs"]
mod cli_kernel_domain_boundaries;

#[path = "architecture/config_architecture_boundaries.rs"]
mod config_architecture_boundaries;

#[path = "architecture/dev_cli_architecture_guards.rs"]
mod dev_cli_architecture_guards;

#[path = "architecture/dev_cli_command_implementation_ownership.rs"]
mod dev_cli_command_implementation_ownership;

#[path = "architecture/dev_cli_invariants_boundaries.rs"]
mod dev_cli_invariants_boundaries;

#[path = "architecture/dev_cli_ownership_boundaries.rs"]
mod dev_cli_ownership_boundaries;

#[path = "architecture/query_interfaces.rs"]
mod query_interfaces;

#[path = "architecture/runtime_query_architecture_boundaries.rs"]
mod runtime_query_architecture_boundaries;
