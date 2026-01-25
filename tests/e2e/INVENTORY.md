# E2E Test Inventory (Iteration 1 Triage)

Labels:
- A — Behavioral System Test (keep & improve)
- B — Regression Duplication (move to regression or delete)
- C — Snapshot / Output Only (rewrite or delete)
- D — Meaningless (delete immediately)

## command_sequences/test_command_sequences.py
- test_invalid_ordering_does_not_corrupt_state: A
- test_valid_ordering_sequences: A
- test_plugin_ordering_list_info_uninstall_then_install: A

## exit_policy/test_exit_policy.py
- test_invalid_inputs_do_not_corrupt_state: A
- test_broken_plugin_metadata_fails_cleanly: A
- test_plugin_missing_metadata_fails: A
- test_plugin_invalid_metadata_fields_fails: A

## config_state/test_config_state.py
- test_config_set_unset_set_again: A
- test_config_unset_preserves_other_keys: A
- test_config_set_is_idempotent: A
- test_config_unset_is_idempotent: A
- test_config_get_repeat_is_stable: A

## plugin_lifecycle/test_plugin_lifecycle.py
- test_plugin_install_uninstall_reinstall: A
- test_plugin_install_is_idempotent_with_force: A
- test_plugin_uninstall_is_idempotent: A

## plugin_lifecycle/test_plugins_smoke.py
- test_plugin_install_load_run: A

## precedence/test_precedence.py
- test_env_overrides_config_value: A
- test_explicit_config_persists_without_env: A
- test_format_json_preserves_exit_code_for_missing_key: A
- test_quiet_suppresses_output_not_exit_code: A
- test_log_level_trace_does_not_change_exit_code: A
- test_quiet_overrides_trace_output: A
- test_format_json_with_trace_preserves_exit_code: A

## api_cli_parity/test_api_cli_parity.py
- test_api_set_cli_get_parity: A
- test_cli_set_api_shares_config: A
- test_api_cli_plugin_lifecycle_parity: A
- test_cli_reinstall_force_api_sees_plugin: A

## Reclassification notes
- The rewrite removed snapshot-only checks; all tests assert state and invariants.
