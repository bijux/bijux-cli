#![forbid(unsafe_code)]
//! Feature modules containing business behavior and policies.

/// Configuration domain logic and command behavior.
pub mod config;
/// Installation compatibility and state management behavior.
pub mod install;
/// Developer command behavior and runtime-query integration.
pub mod developer;
/// History state management and command behavior.
pub mod history;
/// Memory state management and command behavior.
pub mod memory;
/// Plugin discovery, manifest validation, and registry state.
pub mod plugins;
/// Read-only runtime diagnostics query providers.
pub mod diagnostics;
