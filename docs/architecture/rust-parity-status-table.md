# Rust Parity Status Table

| Capture | Command | Status | Exit | Stdout | Stderr | Rust ms |
|---|---|---|---|---|---|---:|
| bijux_help | `--help` | rust-partial | match | diff | match | 528.83 |
| bijux_version | `version` | rust-partial | match | diff | match | 325.23 |
| bijux_doctor | `doctor` | rust-partial | match | diff | match | 310.42 |
| bijux_status_text | `status` | rust-partial | match | diff | match | 302.30 |
| bijux_status_json_no_pretty | `status -f json --no-pretty` | rust-partial | match | diff | match | 394.41 |
| bijux_status_yaml_pretty | `status -f yaml --pretty` | rust-partial | match | diff | match | 312.80 |
| bijux_plugins_list | `plugins list` | rust-partial | match | diff | match | 312.11 |
| bijux_config_root | `config` | rust-partial | match | diff | match | 406.78 |
| bijux_history_root | `history` | rust-partial | match | diff | match | 329.62 |
| bijux_dev_help | `dev --help` | rust-partial | match | diff | match | 302.13 |
| behavior_plugins_check | `plugins check capture_plugin` | rust-partial | match | match | diff | 420.72 |
| behavior_config_precedence_config_only | `config get sample_key` | python-only | diff | diff | diff | 325.84 |
| behavior_config_precedence_env_override | `config get sample_key` | python-only | diff | diff | diff | 318.99 |
| behavior_config_precedence_cli_override | `plugins list --log-level debug` | rust-partial | match | diff | diff | 430.61 |

## Crate Checks

- `bin`: pass
- `core`: pass
- `output`: pass
- `plugin`: pass
- `repl`: pass
- `python`: fail
