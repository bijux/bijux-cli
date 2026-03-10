#![forbid(unsafe_code)]
//! Routing suite module root.

mod command_tree_diff_report;
mod command_tree_snapshot;
mod config_domain_types;
mod dev_cli_inventory_boundaries;
mod dev_cli_routing_identity_boundaries;
mod envelope_compatibility;
mod flag_normalization_property;
mod legacy_forms_regression;
mod normalization_property;
mod parser_abuse;
mod parser_fixtures;
mod parser_fuzz;
mod parser_fuzz_regressions;
mod parser_fuzz_targets;
mod parser_intent;
mod query_interfaces;
mod registry_namespace_policy;
mod registry_resolution;
mod route_fuzz_regressions;
mod route_fuzz_targets;
mod route_inspection_output;
mod route_law_consistency;
mod routing_fixture_sets;
mod schema_snapshots;
mod serde_roundtrip;
