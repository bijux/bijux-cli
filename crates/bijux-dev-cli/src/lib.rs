#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

/// Application-layer command dispatch and runtime query interfaces.
pub mod application;
/// Capability modules organized by maintainer command ownership.
pub mod capabilities;
/// Stable shared catalogs and contracts used across dev-cli capabilities.
pub mod catalog;
/// Reusable technical helpers with no command ownership.
pub mod support;

pub use application::dispatch;
pub use capabilities::automation_contracts as scripts;
pub use capabilities::cockpit;
pub use capabilities::config;
pub use capabilities::control_plane;
pub use capabilities::crate_health;
pub use capabilities::docs_audit;
pub use capabilities::env;
pub use capabilities::evidence;
pub use capabilities::package_health;
pub use capabilities::parity;
pub use capabilities::python;
pub use capabilities::registry;
pub use capabilities::release;
pub use capabilities::repo;
pub use capabilities::route_audit;
pub use capabilities::routes;
pub use capabilities::runtime_contracts as contracts;
pub use capabilities::runtime_identity;
pub use capabilities::rustdoc;
pub use capabilities::script_audit;
pub use capabilities::state_audit;
pub use capabilities::status;
pub use catalog::report_envelope as reporting;

pub use catalog::command_registry::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
