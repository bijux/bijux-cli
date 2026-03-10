#![forbid(unsafe_code)]
//! Snapshot test for rendered command tree roots.

use bijux_cli::routing::registry::RouteRegistry;
use proptest as _;
use serde as _;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn command_tree_snapshot_matches_expected() {
    let registry = RouteRegistry::default();
    let actual = registry.render_command_tree();
    let expected = include_str!("snapshots/command_tree.txt");
    assert_eq!(actual, expected);
}
