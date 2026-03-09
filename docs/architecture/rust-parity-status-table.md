# Rust Parity Status Table

| Capture | Command | Status | Exit | Stdout | Stderr | Rust ms |
|---|---|---|---|---|---|---:|
| bijux_help | `--help` | rust-partial | match | diff | match | 339.16 |
| bijux_version | `version` | rust-partial | match | diff | match | 299.88 |
| bijux_doctor | `doctor` | rust-partial | match | diff | match | 322.05 |
| bijux_status_text | `status` | rust-partial | match | diff | match | 315.94 |
| bijux_status_json_no_pretty | `status -f json --no-pretty` | rust-partial | match | diff | match | 424.12 |
| bijux_status_yaml_pretty | `status -f yaml --pretty` | rust-partial | match | diff | match | 323.37 |
| bijux_plugins_list | `plugins list` | rust-partial | match | diff | match | 323.65 |
| bijux_config_root | `config` | rust-partial | match | diff | match | 403.75 |
| bijux_history_root | `history` | rust-partial | match | diff | match | 308.58 |
| bijux_dev_help | `dev --help` | rust-partial | match | diff | match | 322.95 |
| behavior_plugins_check | `plugins check capture_plugin` | rust-partial | match | diff | diff | 414.37 |

## Crate Checks

- `bin`: pass
- `core`: pass
- `output`: pass
- `plugin`: pass
- `repl`: pass
- `python`: pass
