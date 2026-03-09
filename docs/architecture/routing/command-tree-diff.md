# Rust vs Python Command Tree Diff

Date: 2026-03-09
Inputs:
- `crates/bijux-cli-routing/tests/fixtures/python_documented_commands.txt`
- `crates/bijux-cli-routing/tests/fixtures/rust_routed_root_commands.txt`

## Summary
- Python documented root commands: 16
- Rust routed root commands: 10
- Overlap: 9
- Python-only: 7
- Rust-only: 1

## Overlap
- `completion`
- `config`
- `dev`
- `doctor`
- `inspect`
- `plugins`
- `repl`
- `status`
- `version`

## Python-only (not yet routed as Rust root command)
- `atlas`
- `audit`
- `docs`
- `help`
- `history`
- `memory`
- `sleep`

## Rust-only
- `cli`

## Interpretation
Current Rust routing intentionally focuses on the first parity slice (`version`, `doctor`, `status`, `inspect`, grouped `cli` and `dev cli` diagnostics/config/plugin read paths). Remaining Python roots are tracked as explicit parity gaps.
