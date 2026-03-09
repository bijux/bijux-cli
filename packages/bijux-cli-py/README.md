# bijux-cli-py

Python distribution wrapper for `bijux-cli`.

- Primary runtime path: PyO3 + maturin extension (`bijux_cli_py._native`)
- Fallback path: subprocess delegation to canonical `bijux`/`bijux-rs` runtime
