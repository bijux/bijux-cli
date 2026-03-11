# Stdout Diff

| Command | Match | Python | Rust |
|---|---|---|---|
| `--help` | no | `Usage: bijux [OPTIONS] COMMAND [ARGS]...\n\n  Bijux CLI – Lean, plug-in-driven command-line interface.\n\nOptions:\n  --install-completion  Install completion for the current sh...` | `Usage: bijux [OPTIONS] [COMMAND]\n\nCommands:\n  cli         \n  dev         \n  status      \n  audit       \n  docs        \n  sleep       \n  doctor      \n  version     \...` |
| `version` | no | `{"version": "0.1.3"}\n` | `{\n  "version": "0.1.0"\n}\n` |
| `doctor` | no | `{\n  "status": "healthy",\n  "summary": [\n    "All core checks passed"\n  ],\n  "products": {\n    "atlas": [\n      {\n        "binary": "bijux-atlas",\n        "path": null,\...` | `{\n  "checks": [\n    "routing",\n    "output",\n    "config",\n    "install"\n  ],\n  "install": {\n    "has_duplicate_installs": false,\n    "has_mismatched_wheel_binary_versi...` |
| `status` | no | `{\n  "status": "ok",\n  "products": {\n    "atlas": [\n      {\n        "binary": "bijux-atlas",\n        "path": null,\n        "version": null,\n        "compatible_major": nu...` | `{\n  "runtime": "rust-foundation",\n  "status": "ok"\n}\n` |
| `status -f json --no-pretty` | no | `{\n  "status": "ok",\n  "products": {\n    "atlas": [\n      {\n        "binary": "bijux-atlas",\n        "path": null,\n        "version": null,\n        "compatible_major": nu...` | `{"runtime":"rust-foundation","status":"ok"}\n` |
| `status -f yaml --pretty` | no | `status: ok\nproducts:\n  atlas:\n  - binary: bijux-atlas\n    path: null\n    version: null\n    compatible_major: null\n  - binary: bijux-dev-atlas\n    path: null\n    version...` | `runtime: rust-foundation\nstatus: ok\n` |
| `plugins list` | no | `{\n  "plugins": []\n}\n` | `{\n  "directory": "/Users/bijan/.bijux/.plugins",\n  "plugins": []\n}\n` |
| `config` | no | `{}\n` | `{\n  "BIJUXCLI_CONFIG": "/Users/bijan/.bijux/.env",\n  "BIJUXCLI_HISTORY_FILE": "/Users/bijan/.bijux/.history",\n  "BIJUXCLI_PLUGINS_DIR": "/Users/bijan/.bijux/.plugins"\n}\n` |
| `history` | no | `{\n  "entries": [\n    {\n      "command": "doctor",\n      "params": [],\n      "timestamp": 1773079320.774014,\n      "success": true,\n      "return_code": 0,\n      "duratio...` | `{\n  "entries": [\n    {\n      "command": "dev atlas configs doctor --format text",\n      "duration_ms": 7.0,\n      "params": [\n        "atlas",\n        "configs",\n       ...` |
| `dev --help` | no | `Usage: bijux dev [OPTIONS] COMMAND [ARGS]...\n\n  Developer tools and diagnostics.\n\nOptions:\n  -q, --quiet             Suppress normal output; exit code still indicates\n    ...` | `Usage: bijux dev [OPTIONS] [COMMAND]\n\nCommands:\n  cli   \n  help  Print this message or the help of the given subcommand(s)\n\nOptions:\n  -f, --format <FORMAT>     \n  -q...` |
| `plugins check capture_plugin` | yes | `{\n  "plugin": "capture_plugin",\n  "status": "healthy"\n}\n` | `{\n  "plugin": "capture_plugin",\n  "status": "healthy"\n}\n` |
| `config get sample_key` | no | `{\n  "value": "from_config"\n}\n` | `` |
| `config get sample_key` | no | `{\n  "value": "from_env"\n}\n` | `` |
| `plugins list --log-level debug` | no | `{\n  "plugins": [],\n  "python": "3.11.9",\n  "platform": "macOS-26.3.1-arm64-arm-64bit"\n}\n` | `{\n  "directory": "/Users/bijan/.bijux/.plugins",\n  "plugins": []\n}\n` |
