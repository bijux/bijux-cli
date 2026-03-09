# Rust Parity Status Table

| Capture | Command | Status | Exit | Stdout | Stderr | Rust ms |
|---|---|---|---|---|---|---:|
| bijux_help | `--help` | rust-partial | match | diff | match | 410.82 |
| bijux_version | `version` | rust-partial | match | diff | match | 314.41 |
| bijux_doctor | `doctor` | rust-partial | match | diff | match | 333.99 |
| bijux_status_text | `status` | rust-partial | match | diff | match | 454.43 |
| bijux_status_json_no_pretty | `status -f json --no-pretty` | rust-partial | match | diff | match | 335.44 |
| bijux_status_yaml_pretty | `status -f yaml --pretty` | rust-partial | match | diff | match | 335.92 |
| bijux_plugins_list | `plugins list` | rust-partial | match | diff | match | 346.62 |
| bijux_config_root | `config` | rust-partial | match | diff | match | 328.64 |
| bijux_history_root | `history` | rust-partial | match | diff | match | 335.54 |
| bijux_dev_help | `dev --help` | rust-partial | match | diff | match | 391.56 |
| behavior_plugins_check | `plugins check capture_plugin` | rust-partial | match | match | diff | 328.04 |

## Crate Checks

- `bin`: pass
- `core`: pass
- `output`: pass
- `plugin`: pass
- `repl`: pass
- `python`: pass
