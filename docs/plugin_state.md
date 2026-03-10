# Plugin State

This document is intentionally direct.

## Current Truth

Implemented and stable enough for baseline use:
- `plugins list`
- `plugins inspect`
- `plugins check`
- `plugins reserved-names`
- `plugins where`
- `plugins explain`
- `plugins schema`

Still partial in command-surface parity:
- `plugins scaffold`
- `plugins install`
- `plugins uninstall`
- `plugins enable`
- `plugins disable`

## Beyond Python Today

Areas where Rust plugin behavior already provides stronger guardrails:
- explicit reserved-namespace diagnostics surfaces
- plugin origin metadata in inspection paths
- transaction rollback assertions for write-path failure cases
- schema discoverability via `plugins schema`

## Overlap Parity Evidence

- `crates/bijux-cli-plugin/tests/plugin_parity_read_paths.rs`
- `crates/bijux-cli/tests/bin_surface/plugin_command_parity.rs`
- `artifacts/status/plugin_state_report.json`

## Known Gaps

- scaffold fixture parity against Python templates is incomplete
- full CLI parity for install/uninstall/enable/disable command surfaces is incomplete
- full end-to-end lifecycle parity across all failure classes is incomplete

## Plugin V1 Law Freeze

Plugin v1 behavior is frozen before adding new command cleverness. New plugin features must land with parity evidence and rollback/failure coverage.
