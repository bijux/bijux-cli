# Parity Decision Log

Date: 2026-03-09

## Decision scope

This log records mismatch disposition for the currently covered Rust-vs-Python command set.

## Accepted mismatches (temporary)

1. Help text formatting differs while preserving command intent and discoverability.
2. Structured payload field naming differs for some root commands where Rust currently emits stable contract-first envelopes.
3. Python plugin diagnostics include plugin-loader stderr lines not currently emitted by Rust.

## Bugs fixed in this wave

1. `config` root no longer exits as unknown route.
2. `history` root no longer exits as unknown route.
3. `plugins check <plugin>` no longer fails parse due unsupported trailing argument.

## Enforcement rule

No mismatch can move from `rust-complete` to `rust-partial` or `python-only` without explicit baseline update in `docs/architecture/parity/baseline-parity-v1.json`.
