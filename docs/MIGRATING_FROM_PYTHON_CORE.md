# Migrating from Python Core to Rust Foundation

## Purpose
Define the migration path from the Python implementation to the Rust foundation while preserving compatibility.

## Compatibility principle
The `bijux` command is the product surface. Migration work must preserve documented contracts for command grammar, flags, output envelopes, errors, stream routing, and exit codes.

## Current baseline inputs
- Contracts and compatibility guarantees under `docs/07-contracts/`.
- Python-facing compatibility baseline under `docs/06-reference/integrations-and-routed-runtimes.md`.
- Current Rust vs current Python parity tests.
- Stable-release compatibility checks against the published PyPI baseline.

## Migration stages
1. Implement Rust contracts and typed models in `bijux-cli-contracts`.
2. Implement parser and route graph in `bijux-cli`.
3. Implement execution kernel in `bijux-cli`.
4. Implement output and error emission in `bijux-cli-output`.
5. Implement plugin and REPL boundaries in dedicated crates.
6. Run parity checks against the current Python package and the stable PyPI baseline.
7. Cut over entrypoint only after parity criteria pass.

## Parity gates
- Root command help and command graph parity.
- Stable global flag behavior parity.
- Stable output envelope parity in JSON/YAML modes.
- Stable error envelope and failure-class parity.
- Stable exit-code parity for success, usage, validation, plugin, and internal errors.
- Stable stdout/stderr routing parity.

## Cutover strategy
- Keep `pip install bijux-cli` as the primary install path.
- Ship Rust internals behind compatibility-preserving release boundaries.
- If no documented behavior breaks, release as a minor version.
- If documented behavior changes incompatibly, release as a major version.

## Rollback strategy
- Keep rollback instructions tied to released package versions, not to checked-in
  generated capture files.
- On parity regression, revert to the last known-good published version in the
  affected channel.
