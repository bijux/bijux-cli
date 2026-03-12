# Rust vs Python Command Tree Diff

Date: 2026-03-12
Inputs:
- `crates/bijux-cli/tests/data/fixtures/routing/python_documented_commands.txt`
- `crates/bijux-cli/tests/data/fixtures/routing/rust_routed_root_commands.txt`

## Summary
- Python documented root commands: 16
- Rust routed root commands: 16
- Overlap: 15
- Python-only: 1
- Rust-only: 1

## Overlap
- `audit`
- `completion`
- `config`
- `dev`
- `docs`
- `doctor`
- `help`
- `history`
- `inspect`
- `memory`
- `plugins`
- `repl`
- `sleep`
- `status`
- `version`

## Python-only (not yet routed as Rust root command)
- `atlas`

## Rust-only
- `cli`

## Interpretation
Rust routing now reflects the runtime-owned command surface and delegated roots, while product namespaces such as `atlas` stay reserved in registry policy instead of appearing as runtime command-tree roots.
