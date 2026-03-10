# Rust vs Python Command Tree Diff

Date: 2026-03-10
Inputs:
- `crates/bijux-cli-routing/tests/fixtures/python_documented_commands.txt`
- `crates/bijux-cli-routing/tests/fixtures/rust_routed_root_commands.txt`

## Summary
- Python documented root commands: 16
- Rust routed root commands: 13
- Overlap: 12
- Python-only: 4
- Rust-only: 1

## Overlap
- `completion`
- `config`
- `dev`
- `doctor`
- `atlas`
- `inspect`
- `history`
- `memory`
- `plugins`
- `repl`
- `status`
- `version`

## Python-only (not yet routed as Rust root command)
- `audit`
- `docs`
- `help`
- `sleep`

## Rust-only
- `cli`

## Interpretation
Current Rust routing intentionally focuses on the first parity slice (`version`, `doctor`, `status`, `inspect`, grouped `cli` and `dev cli` diagnostics/config/plugin read paths). Remaining Python roots are tracked as explicit parity gaps.
