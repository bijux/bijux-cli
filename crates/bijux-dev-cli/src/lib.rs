#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

/// Application-layer command dispatch and runtime query interfaces.
pub mod application;
/// Stable shared catalogs and contracts used across dev-cli capabilities.
pub mod catalog;
/// Maintainer-facing command report modules organized by feature ownership.
pub mod features;
/// Reusable technical adapters for filesystem/process/clock concerns.
pub mod infrastructure;
/// Backward-compatibility shim for older `support` imports.
pub mod support;
/// Contract inventories and execution boundaries.
#[path = "contracts/mod.rs"]
pub mod contract_engine;

pub use application::dispatch;
pub use catalog::report_envelope as reporting;
pub use contract_engine::maintenance as scripts;
pub use features::cockpit;
pub use features::config;
pub use features::control_plane;
pub use features::crate_health;
pub use features::docs_audit;
pub use features::env;
pub use features::evidence;
pub use features::package_health;
pub use features::parity;
pub use features::python;
pub use features::registry;
pub use features::release;
pub use features::repo;
pub use features::route_audit;
pub use features::routes;
pub use features::runtime_contracts as contracts;
pub use features::runtime_identity;
pub use features::rustdoc;
pub use features::script_audit;
pub use features::state_audit;
pub use features::status;

pub use catalog::command_registry::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
