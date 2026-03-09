# Rust Parity Status Table

| Capture | Command | Status | Exit | Stdout | Stderr | Rust ms |
|---|---|---|---|---|---|---:|
| bijux_help | `--help` | rust-partial | match | diff | match | 657.73 |
| bijux_version | `version` | rust-partial | match | diff | match | 321.95 |
| bijux_doctor | `doctor` | rust-partial | match | diff | match | 455.97 |
| bijux_status_text | `status` | rust-partial | match | diff | match | 331.71 |
| bijux_status_json_no_pretty | `status -f json --no-pretty` | rust-partial | match | diff | match | 334.86 |
| bijux_status_yaml_pretty | `status -f yaml --pretty` | rust-partial | match | diff | match | 347.47 |
| bijux_plugins_list | `plugins list` | rust-partial | match | diff | match | 332.28 |
| bijux_config_root | `config` | rust-partial | match | diff | match | 331.33 |
| bijux_history_root | `history` | rust-partial | match | diff | match | 400.91 |
| bijux_dev_help | `dev --help` | rust-partial | match | diff | match | 339.69 |
| behavior_plugins_check | `plugins check capture_plugin` | rust-partial | match | match | diff | 318.14 |
| behavior_config_precedence_config_only | `config get sample_key` | python-only | diff | diff | diff | 387.43 |
| behavior_config_precedence_env_override | `config get sample_key` | python-only | diff | diff | diff | 337.00 |
| behavior_config_precedence_cli_override | `plugins list --log-level debug` | rust-partial | match | diff | diff | 340.56 |

## Crate Checks

- `bin`: pass
- `core`: pass
- `output`: pass
- `plugin`: pass
- `repl`: pass
- `python`: fail
