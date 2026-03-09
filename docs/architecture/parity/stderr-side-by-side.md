# Stderr Diff

| Command | Match | Python | Rust |
|---|---|---|---|
| `--help` | yes | `` | `` |
| `version` | yes | `` | `` |
| `doctor` | yes | `` | `` |
| `status` | yes | `` | `` |
| `status -f json --no-pretty` | yes | `` | `` |
| `status -f yaml --pretty` | yes | `` | `` |
| `plugins list` | yes | `` | `` |
| `config` | yes | `` | `` |
| `history` | yes | `` | `` |
| `dev --help` | yes | `` | `` |
| `plugins check capture_plugin` | no | `Plugin capture_plugin failed to register: Plugin has no CLI entrypoint (expected cli() or app)\nPlugin capture_plugin failed to register: Plugin has no CLI entrypoint (expected ...` | `` |
| `config get sample_key` | no | `` | `{\n  "code": 2,\n  "command": "cli config get",\n  "message": "Config key not found: config",\n  "status": "error"\n}\n` |
| `config get sample_key` | no | `` | `{\n  "code": 2,\n  "command": "cli config get",\n  "message": "Config key not found: config",\n  "status": "error"\n}\n` |
| `plugins list --log-level debug` | no | `DIContainer initialised\nDIContainer.current auto-initialized singleton\nRegistered service\nRegistered service\nRegistered service\nRegistered service\nRegistered service\nRegi...` | `` |
