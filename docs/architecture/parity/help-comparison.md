# Help Comparison

Python capture artifacts do not include per-command `--help` outputs for the selected newly ported commands.
Rust `--help` output is validated via existing help snapshot tests in `crates/bijux-cli-bin/tests/help_snapshots.rs`.
