# Help Comparison

Python capture artifacts do not include complete per-command `--help` captures for every routed Rust command.

Rust help behavior is validated by snapshot and stability tests in:

- `crates/bijux-cli-bin/tests/help_snapshots.rs`

Detailed parity decisions and accepted deltas are tracked in:

- `docs/architecture/help-parity-report.md`
- `docs/architecture/help-rendering-rules.md`
