# Rust vs Python Command Tree Diff

Date: 2026-03-09
Inputs:
- `crates/bijux-cli-routing/tests/fixtures/python_documented_commands.txt`
- `crates/bijux-cli-routing/tests/fixtures/rust_routed_root_commands.txt`

## Summary
- Python documented root commands: 16
- Rust routed root commands: 12
- Overlap: 11
- Python-only: 5
- Rust-only: 1

## Overlap
- `completion`
- `config`
- `dev`
- `doctor`
- `inspect`
- `history`
- `memory`
- `plugins`
- `repl`
- `status`
- `version`

## Python-only (not yet routed as Rust root command)
- `atlas`
- `audit`
- `docs`
- `help`
- `sleep`

## Rust-only
- `cli`

## Interpretation
Current Rust routing intentionally focuses on the first parity slice (`version`, `doctor`, `status`, `inspect`, grouped `cli` and `dev cli` diagnostics/config/plugin read paths). Remaining Python roots are tracked as explicit parity gaps.
