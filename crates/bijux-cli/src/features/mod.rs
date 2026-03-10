#![forbid(unsafe_code)]
//! Feature modules containing business behavior and policies.

/// Configuration domain logic and command behavior.
pub mod config;
/// Installation compatibility and state management behavior.
pub mod install;
/// Plugin discovery, manifest validation, and registry state.
pub mod plugins;
/// Read-only runtime diagnostics query providers.
pub mod diagnostics;
