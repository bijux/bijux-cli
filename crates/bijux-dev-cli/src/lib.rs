#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

/// Application building blocks for argument parsing, workspace resolution, and routing.
pub mod app;
/// Application-layer command dispatch and runtime query interfaces.
pub mod application;
/// Stable shared catalogs and contracts used across dev-cli capabilities.
pub mod catalog;
/// Maintainer-facing command report modules organized by command intent.
pub mod commands;
/// Contract inventories and execution boundaries.
#[path = "contracts/mod.rs"]
pub mod contract_engine;
/// Compatibility aliases for feature-oriented module paths.
pub mod features;
/// Reusable technical adapters for filesystem/process/clock concerns.
pub mod infrastructure;
/// Status contract inventory and execution services.
pub mod status_contracts;

pub use application::dispatch;
pub use catalog::report_envelope as reporting;
pub use commands::cockpit;
pub use commands::config;
pub use commands::crate_health;
pub use commands::docs_audit;
pub use commands::env;
pub use commands::evidence;
pub use commands::package_health;
pub use commands::parity;
pub use commands::python;
pub use commands::registry;
pub use commands::release;
pub use commands::repo;
pub use commands::route_audit;
pub use commands::routes;
pub use commands::runtime_contracts as contracts;
pub use commands::runtime_identity;
pub use commands::rustdoc;
pub use commands::script_audit;
pub use commands::state_audit;
pub use commands::status;
pub use contract_engine::maintenance as scripts;
pub use features::control_plane;

pub use catalog::command_registry::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
