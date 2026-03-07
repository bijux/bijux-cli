# CLI Deprecation and Alias Policy

## Scope

Defines how command aliases and deprecations are introduced and governed for `bijux`.

## Alias rules

- Aliases must be explicit and documented in command taxonomy and CLI contract docs.
- Aliases must preserve exit-code class and JSON envelope compatibility.
- Aliases must not silently narrow validation behavior.

## Deprecation rules

- A deprecated command must keep working for at least one documented compatibility window.
- Deprecation requires:
  - replacement command
  - migration note
  - release-note entry
  - contract test coverage
- Deprecated surfaces must emit stable diagnostics in both human and JSON modes.

## Current aliases

- `dag fsck <run-dir>` is a stable alias surface for run integrity verification (`dag verify <run-dir> --deep`).

## Tests

- `crates/bijux-dag-cli/tests/contract_surface.rs`
- `crates/bijux-dag-app/tests/cli_contract.rs`
