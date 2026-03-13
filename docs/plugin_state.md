# Plugin State

This document is intentionally direct.

## Current Truth

Implemented and stable enough for baseline use:
- `plugins list`
- `plugins inspect`
- `plugins check`
- `plugins enable`
- `plugins disable`
- `plugins install`
- `plugins uninstall`
- `plugins scaffold`
- `plugins doctor`
- `plugins reserved-names`
- `plugins where`
- `plugins explain`
- `plugins schema`

Still intentionally bounded:
- plugin lifecycle commands are implemented, but installed plugin namespaces are not yet executed as
  runtime subcommands inside `bijux-cli`

## Beyond Python Today

Areas where Rust plugin behavior already provides stronger guardrails:
- explicit reserved-namespace diagnostics surfaces
- plugin origin metadata in inspection paths
- transaction rollback assertions for write-path failure cases
- schema discoverability via `plugins schema`
- delegated entrypoint presence checks for local manifest installs

## Overlap Parity Evidence

- `crates/bijux-cli/tests/integration/cli/plugins/plugin_cli_lifecycle.rs`
- `crates/bijux-cli/tests/integration/cli/plugins/plugin_command_parity.rs`
- `artifacts/status/plugin_state_report.json`

## Known Gaps

- installed plugin command execution remains outside the current runtime scope
- template and scaffold surfaces must keep tracking the same release window and manifest contract

## Plugin V1 Law Freeze

Plugin v1 behavior is frozen before adding new command cleverness. New plugin features must land with parity evidence and rollback/failure coverage.
