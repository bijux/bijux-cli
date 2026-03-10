# Python Bridge Library Audit

Date: 2026-03-09

## Scope

Audited crate: `crates/bijux-cli-python/src/lib.rs` and module split.

## Result

The bridge was split into focused modules:

- `bindings.rs`: bridge APIs and command execution adapters.
- `conversions.rs`: error-kind and exception-tag classification.
- `compatibility.rs`: compatibility filesystem/config APIs re-exported from install crate.

## Key architectural decisions

1. The bridge now executes commands through `bijux_cli::app::run_app`, matching the binary entrypoint behavior.
2. Compatibility domain logic was moved out of the bridge runtime path and shared through `bijux-cli::install`.
3. The bridge crate remains a thin API adapter instead of a second command-law implementation.

## Residual risks

1. Error classification currently uses a stable coarse mapping (`UsageError`, `ValidationError`, `InternalError`) and may need stricter envelope categories later.
2. Bridge API returns JSON payload strings for compatibility; typed Python wrapper evolution is still open.
