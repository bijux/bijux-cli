//! `cli` command handlers.

use std::env;
use std::path::Path;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};

use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::features::install::{install_health_report, post_install_hint};
use crate::features::plugins::{compatibility_warnings, plugin_origin_metadata};
use crate::routing::registry::RouteRegistry;

pub(crate) fn try_handle(
    normalized_path: &[String],
    paths: &ResolvedStatePaths,
    registry: &RouteRegistry,
    plugin_registry_path: &Path,
) -> Option<Value> {
    match normalized_path {
        [a, b] if a == "cli" && b == "version" => {
            Some(json!({"version": env!("CARGO_PKG_VERSION")}))
        }
        [a, b] if a == "cli" && b == "doctor" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            Some(json!({
                "status": "healthy",
                "checks": ["routing", "output", "config", "install"],
                "install": {
                    "has_path_shadowing": install_report.has_path_shadowing,
                    "has_duplicate_installs": install_report.has_duplicate_installs,
                    "stale_wrapper_scripts": install_report.stale_wrapper_scripts,
                    "legacy_installer_conflicts": false,
                    "has_mismatched_wheel_binary_versions": install_report.has_mismatched_wheel_binary_versions,
                }
            }))
        }
        [a, b] if a == "cli" && b == "repl" => {
            Some(json!({"status": "ready", "mode": "repl", "history_file": paths.history_file}))
        }
        [a, b] if a == "cli" && b == "completion" => {
            Some(json!({"shells": ["bash", "zsh", "fish", "powershell"]}))
        }
        [a, b] if a == "cli" && b == "inspect" => {
            let route_sources: Vec<Value> = registry
                .built_in_paths()
                .into_iter()
                .map(|path| {
                    let segments: Vec<String> = path.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "segments": segments,
                        "owner": "bijux-cli",
                        "source": "built-in",
                    })
                })
                .collect();
            Some(json!({
                "status": "ok",
                "reserved_namespaces": registry.route_tree(),
                "builtins": registry.built_in_paths(),
                "route_sources": route_sources,
                "alias_rewrites": registry.alias_rewrites().into_iter().map(|(alias, canonical)| {
                    let alias_segments: Vec<String> = alias.segments.into_iter().map(|s| s.0).collect();
                    let canonical_segments: Vec<String> = canonical.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "alias": alias_segments,
                        "canonical": canonical_segments,
                        "source": "compatibility-alias",
                    })
                }).collect::<Vec<_>>(),
                "plugin_origins": plugin_origin_metadata(plugin_registry_path).unwrap_or_default(),
                "compatibility_warnings": compatibility_warnings(plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
                "contracts": {
                    "schemas": ["output-envelope-v1", "error-envelope-v1", "plugin-manifest-v1"],
                    "version": "v1",
                }
            }))
        }
        [a, b] if a == "cli" && b == "status" => {
            Some(json!({"status": "ok", "runtime": "rust-foundation"}))
        }
        [a, b] if a == "cli" && b == "paths" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let hint = install_report
                .active_binary
                .as_deref()
                .map(post_install_hint)
                .unwrap_or_else(|| {
                    "Run `bijux version` and `bijux cli doctor` to verify your environment."
                        .to_string()
                });
            Some(json!({
                "config": paths.config_file,
                "history": paths.history_file,
                "plugins": paths.plugins_dir,
                "active_binary": install_report.active_binary,
                "path_binaries": install_report.path_binaries,
                "post_install_hint": hint
            }))
        }
        [a, b] if a == "cli" && b == "self-test" => {
            Some(json!({"status": "ok", "checks": ["routing", "contracts", "emitters"]}))
        }
        [a, b, c] if a == "cli" && b == "hold" && c == "interruptible" => {
            for _ in 0..200_u16 {
                thread::sleep(Duration::from_millis(50));
            }
            Some(json!({"status": "completed"}))
        }
        _ => None,
    }
}
