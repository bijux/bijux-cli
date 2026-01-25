# E2E Inventory

One row per E2E test file. Primary intent is the main reason the test exists.

| Test file | Primary intent | Secondary intent | Invariants | Notes |
| --- | --- | --- | --- | --- |
| tests/e2e/command_sequences/test_command_sequences.py | compositional | ordering | config consistency, exit stability, no corruption | restart-driven sequences |
| tests/e2e/config_state/test_config_state.py | stateful | idempotence | config consistency, no corruption | reversible config changes |
| tests/e2e/precedence/test_precedence.py | precedence | exit_policy | exit stability, config consistency | env/flag precedence |
| tests/e2e/exit_policy/test_exit_policy.py | exit_policy | failure | no corruption, exit stability | error routing |
| tests/e2e/plugin_lifecycle/test_plugin_lifecycle.py | plugin | idempotence | plugin consistency, exit stability | abusive lifecycle sequences |
| tests/e2e/plugin_lifecycle/test_plugins_smoke.py | plugin | compositional | plugin consistency | packaging + entry points |
| tests/e2e/api_cli_parity/test_api_cli_parity.py | api_parity | compositional | config consistency, plugin consistency | CLI/API parity |
