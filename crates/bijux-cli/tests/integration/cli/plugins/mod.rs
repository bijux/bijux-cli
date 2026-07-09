#![forbid(unsafe_code)]
//! plugins integration suites.

use semver::{Prerelease, Version};

mod plugin_cli_lifecycle;
mod plugin_command_parity;
mod plugin_discovery_determinism_matrix;
mod plugin_failure_injection;
mod plugin_failure_rollback_matrix;
mod plugin_lifecycle_matrix;
mod plugin_namespace_law;
mod plugin_scaffold_case_replays;
mod plugin_scaffold_minimal;
mod plugin_scaffold_stability;

pub(super) fn current_plugin_host_floor() -> String {
    let runtime = Version::parse(env!("CARGO_PKG_VERSION")).expect("package semver");
    let mut floor = Version::new(runtime.major, runtime.minor, runtime.patch);
    if !runtime.pre.is_empty() {
        let channel = runtime.pre.as_str().split('.').next().expect("runtime prerelease channel");
        floor.pre = Prerelease::new(channel).expect("prerelease channel");
    }
    floor.to_string()
}

pub(super) fn current_plugin_host_ceiling() -> String {
    let runtime = Version::parse(env!("CARGO_PKG_VERSION")).expect("package semver");
    if runtime.major == 0 {
        Version::new(0, runtime.minor.saturating_add(1), 0).to_string()
    } else {
        Version::new(runtime.major.saturating_add(1), 0, 0).to_string()
    }
}
