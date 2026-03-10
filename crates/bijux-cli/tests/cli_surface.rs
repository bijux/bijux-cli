#![forbid(unsafe_code)]
//! CLI surface integration suites hosted directly in bijux-cli.

#[path = "cli_surface/app_direct_invocation.rs"]
mod app_direct_invocation;

#[path = "cli_surface/config_get_performance.rs"]
mod config_get_performance;

#[path = "cli_surface/config_key_value_parity.rs"]
mod config_key_value_parity;

#[path = "cli_surface/config_core_parity.rs"]
mod config_core_parity;

#[path = "cli_surface/config_root_listing.rs"]
mod config_root_listing;

#[path = "cli_surface/adversarial_fs_process_campaign_regressions.rs"]
mod adversarial_fs_process_campaign_regressions;

#[path = "cli_surface/adversarial_fs_process_campaigns.rs"]
mod adversarial_fs_process_campaigns;

#[path = "cli_surface/bin_core_integration.rs"]
mod bin_core_integration;

#[path = "cli_surface/cli_command_matrix.rs"]
mod cli_command_matrix;

#[path = "cli_surface/command_execution.rs"]
mod command_execution;

#[path = "cli_surface/command_family_consistency_extra.rs"]
mod command_family_consistency_extra;

#[path = "cli_surface/config_corruption_campaign_regressions.rs"]
mod config_corruption_campaign_regressions;

#[path = "cli_surface/config_corruption_hardening.rs"]
mod config_corruption_hardening;

#[path = "cli_surface/config_deep_behavior_matrix.rs"]
mod config_deep_behavior_matrix;

#[path = "cli_surface/config_export_load_parity.rs"]
mod config_export_load_parity;

#[path = "cli_surface/config_fuzz_regressions.rs"]
mod config_fuzz_regressions;

#[path = "cli_surface/config_fuzz_targets.rs"]
mod config_fuzz_targets;

#[path = "cli_surface/config_get_parity.rs"]
mod config_get_parity;

#[path = "cli_surface/config_mutation_matrix.rs"]
mod config_mutation_matrix;

#[path = "cli_surface/config_mutation_parity.rs"]
mod config_mutation_parity;

#[path = "cli_surface/config_parity.rs"]
mod config_parity;

#[path = "cli_surface/config_python_compatibility.rs"]
mod config_python_compatibility;

#[path = "cli_surface/config_read_matrix.rs"]
mod config_read_matrix;

#[path = "cli_surface/config_root_parity.rs"]
mod config_root_parity;

#[path = "cli_surface/config_set_parity.rs"]
mod config_set_parity;

#[path = "cli_surface/config_source_precedence_matrix.rs"]
mod config_source_precedence_matrix;

#[path = "cli_surface/cross_command_consistency_matrix.rs"]
mod cross_command_consistency_matrix;

#[path = "cli_surface/cross_surface_equivalence.rs"]
mod cross_surface_equivalence;

#[path = "cli_surface/cross_surface_state_extra.rs"]
mod cross_surface_state_extra;

#[path = "cli_surface/deterministic_hostile_state_matrix.rs"]
mod deterministic_hostile_state_matrix;

#[path = "cli_surface/deterministic_output_matrix.rs"]
mod deterministic_output_matrix;

#[path = "cli_surface/dev_cli_audit_command_contracts.rs"]
mod dev_cli_audit_command_contracts;

#[path = "cli_surface/dev_cli_cockpit_contracts.rs"]
mod dev_cli_cockpit_contracts;

#[path = "cli_surface/dev_cli_command_matrix.rs"]
mod dev_cli_command_matrix;

#[path = "cli_surface/dev_cli_config_ownership_contracts.rs"]
mod dev_cli_config_ownership_contracts;

#[path = "cli_surface/dev_cli_control_plane_closure.rs"]
mod dev_cli_control_plane_closure;

#[path = "cli_surface/dev_cli_dispatch_boundaries.rs"]
mod dev_cli_dispatch_boundaries;

#[path = "cli_surface/dev_cli_evidence_contracts.rs"]
mod dev_cli_evidence_contracts;

#[path = "cli_surface/dev_cli_invariants.rs"]
mod dev_cli_invariants;

#[path = "cli_surface/dev_cli_output_contracts.rs"]
mod dev_cli_output_contracts;

#[path = "cli_surface/dev_cli_parity_closure_contracts.rs"]
mod dev_cli_parity_closure_contracts;

#[path = "cli_surface/dev_cli_python_sovereignty_contracts.rs"]
mod dev_cli_python_sovereignty_contracts;

#[path = "cli_surface/dev_cli_query_interface_parity.rs"]
mod dev_cli_query_interface_parity;

#[path = "cli_surface/dev_cli_repo_docs_scripts_crate_health_contracts.rs"]
mod dev_cli_repo_docs_scripts_crate_health_contracts;

#[path = "cli_surface/dev_cli_repo_health_contracts.rs"]
mod dev_cli_repo_health_contracts;

#[path = "cli_surface/dev_cli_resilience_determinism_contracts.rs"]
mod dev_cli_resilience_determinism_contracts;

#[path = "cli_surface/dev_cli_route_registry_env_contracts.rs"]
mod dev_cli_route_registry_env_contracts;

#[path = "cli_surface/dev_cli_runtime_package_contracts.rs"]
mod dev_cli_runtime_package_contracts;

#[path = "cli_surface/dev_cli_stale_artifact_hardening.rs"]
mod dev_cli_stale_artifact_hardening;

#[path = "cli_surface/dev_cli_state_diagnostics_contracts.rs"]
mod dev_cli_state_diagnostics_contracts;

#[path = "cli_surface/dev_cli_summary_surface_contracts.rs"]
mod dev_cli_summary_surface_contracts;

#[path = "cli_surface/diagnostics_command_matrix.rs"]
mod diagnostics_command_matrix;

#[path = "cli_surface/diagnostics_contract_consistency.rs"]
mod diagnostics_contract_consistency;

#[path = "cli_surface/diagnostics_deep_behavior_extra.rs"]
mod diagnostics_deep_behavior_extra;

#[path = "cli_surface/diagnostics_parity.rs"]
mod diagnostics_parity;

#[path = "cli_surface/diagnostics_snapshots.rs"]
mod diagnostics_snapshots;

#[path = "cli_surface/diagnostics_trust_law_extra.rs"]
mod diagnostics_trust_law_extra;

#[path = "cli_surface/exit_code_law_matrix.rs"]
mod exit_code_law_matrix;

#[path = "cli_surface/flag_normalization_matrix.rs"]
mod flag_normalization_matrix;

#[path = "cli_surface/help_snapshots.rs"]
mod help_snapshots;

#[path = "cli_surface/help_tree_law_extra.rs"]
mod help_tree_law_extra;

#[path = "cli_surface/history_command_matrix.rs"]
mod history_command_matrix;

#[path = "cli_surface/history_deep_behavior_extra.rs"]
mod history_deep_behavior_extra;

#[path = "cli_surface/history_memory_resilience_hardening.rs"]
mod history_memory_resilience_hardening;

#[path = "cli_surface/history_parity.rs"]
mod history_parity;

#[path = "cli_surface/history_write_resilience.rs"]
mod history_write_resilience;

#[path = "cli_surface/install_ambiguity_hardening.rs"]
mod install_ambiguity_hardening;

#[path = "cli_surface/maintainer_control_plane.rs"]
mod maintainer_control_plane;

#[path = "cli_surface/memory_command_matrix.rs"]
mod memory_command_matrix;

#[path = "cli_surface/memory_deep_behavior_extra.rs"]
mod memory_deep_behavior_extra;

#[path = "cli_surface/memory_parity.rs"]
mod memory_parity;

#[path = "cli_surface/metadata_inspection_matrix.rs"]
mod metadata_inspection_matrix;

#[path = "cli_surface/parser_invalid_utf8_argv.rs"]
mod parser_invalid_utf8_argv;

#[path = "cli_surface/performance_realism_hardening.rs"]
mod performance_realism_hardening;

#[path = "cli_surface/plugin_cli_lifecycle.rs"]
mod plugin_cli_lifecycle;

#[path = "cli_surface/plugin_command_parity.rs"]
mod plugin_command_parity;

#[path = "cli_surface/plugin_discovery_determinism_matrix.rs"]
mod plugin_discovery_determinism_matrix;

#[path = "cli_surface/plugin_failure_injection.rs"]
mod plugin_failure_injection;

#[path = "cli_surface/plugin_failure_rollback_matrix.rs"]
mod plugin_failure_rollback_matrix;

#[path = "cli_surface/plugin_lifecycle_matrix.rs"]
mod plugin_lifecycle_matrix;

#[path = "cli_surface/plugin_namespace_law.rs"]
mod plugin_namespace_law;

#[path = "cli_surface/plugin_scaffold_fuzz_regressions.rs"]
mod plugin_scaffold_fuzz_regressions;

#[path = "cli_surface/plugin_scaffold_fuzz_targets.rs"]
mod plugin_scaffold_fuzz_targets;

#[path = "cli_surface/plugin_scaffold_minimal.rs"]
mod plugin_scaffold_minimal;

#[path = "cli_surface/plugin_state_corruption_campaign_regressions.rs"]
mod plugin_state_corruption_campaign_regressions;

#[path = "cli_surface/precedence_matrix.rs"]
mod precedence_matrix;

#[path = "cli_surface/python_command_port_parity.rs"]
mod python_command_port_parity;

#[path = "cli_surface/randomized_config_corruption_campaigns.rs"]
mod randomized_config_corruption_campaigns;

#[path = "cli_surface/randomized_plugin_state_corruption_campaigns.rs"]
mod randomized_plugin_state_corruption_campaigns;

#[path = "cli_surface/randomized_state_corruption_harness.rs"]
mod randomized_state_corruption_harness;

#[path = "cli_surface/randomized_state_corruption_regressions.rs"]
mod randomized_state_corruption_regressions;

#[path = "cli_surface/repl_completion_extra.rs"]
mod repl_completion_extra;

#[path = "cli_surface/repl_execution_law_extra.rs"]
mod repl_execution_law_extra;

#[path = "cli_surface/repl_hostile_session_extra.rs"]
mod repl_hostile_session_extra;

#[path = "cli_surface/repl_hostile_session_hardening.rs"]
mod repl_hostile_session_hardening;

#[path = "cli_surface/repl_startup_performance_budget.rs"]
mod repl_startup_performance_budget;

#[path = "cli_surface/root_command_matrix.rs"]
mod root_command_matrix;

#[path = "cli_surface/state_race_campaign_regressions.rs"]
mod state_race_campaign_regressions;

#[path = "cli_surface/state_race_campaigns.rs"]
mod state_race_campaigns;

#[path = "cli_surface/stream_discipline_matrix.rs"]
mod stream_discipline_matrix;

#[path = "cli_surface/transcript_cases.rs"]
mod transcript_cases;

#[path = "cli_surface/transcript_parity.rs"]
mod transcript_parity;
