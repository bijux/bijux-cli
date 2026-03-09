# Golden Outputs and Behavior Captures

## Purpose
Provide concrete capture files for tasks 41-60 from the current Python implementation.

## Capture root
- `artifacts/python-behavior/golden/*.json`
- `artifacts/python-behavior/runtime/repl-interactive.txt`
- `artifacts/current-python-behavior-lock.json`

## Reproduction command
- `python3 scripts/capture_python_behavior.py`

## Task mapping
- 41: `golden/bijux_help.json`
- 42: `golden/bijux_version.json`
- 43: `golden/bijux_doctor.json`
- 44: `golden/bijux_status_text.json`
- 45: `golden/bijux_status_json_no_pretty.json`
- 46: `golden/bijux_status_yaml_pretty.json`
- 47: `golden/bijux_plugins_list.json`
- 48: `golden/bijux_config_root.json`
- 49: `golden/bijux_history_root.json`
- 50: `golden/bijux_dev_help.json`
- 51: `golden/behavior_success_streams.json`
- 52: `golden/behavior_validation_failure_streams.json`
- 53: `golden/behavior_internal_failure_streams.json`
- 54: `golden/behavior_quiet_mode.json`
- 55: `golden/behavior_debug_log_level.json`
- 56: `golden/behavior_help_short_circuit.json`
- 57: `golden/behavior_repl_startup_piped.json` and `runtime/repl-interactive.txt`
- 58: `golden/behavior_plugins_install.json`, `golden/behavior_plugins_check.json`, `golden/behavior_plugins_uninstall.json`
- 59: `golden/behavior_config_precedence_config_only.json`, `golden/behavior_config_precedence_env_override.json`, `golden/behavior_config_precedence_cli_override.json`
- 60: `artifacts/current-python-behavior-lock.json`

## Format of each golden capture
Each `golden/*.json` file includes:
- `argv`
- `exit_code`
- `stdout`
- `stderr`
- `env_overrides`
