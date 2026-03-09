# Exit Code Diff

| Command | Match | Python | Rust |
|---|---|---|---|
| `--help` | yes | `0` | `0` |
| `version` | yes | `0` | `0` |
| `doctor` | yes | `0` | `0` |
| `status` | yes | `0` | `0` |
| `status -f json --no-pretty` | yes | `0` | `0` |
| `status -f yaml --pretty` | yes | `0` | `0` |
| `plugins list` | yes | `0` | `0` |
| `config` | yes | `0` | `0` |
| `history` | yes | `0` | `0` |
| `dev --help` | yes | `0` | `0` |
| `plugins check capture_plugin` | yes | `0` | `0` |
| `config get sample_key` | no | `0` | `2` |
| `config get sample_key` | no | `0` | `2` |
| `plugins list --log-level debug` | yes | `0` | `0` |
