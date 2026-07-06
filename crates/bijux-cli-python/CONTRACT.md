# bijux-cli-python contract

Responsibility: Python packaging, native bridge, and runtime parity boundary for the `bijux` command runtime.

## Scope

`bijux-cli-python` owns Python distribution metadata, launcher entrypoints,
native binding conversions, compatibility checks, and parity enforcement between
Python callers and the native `bijux` runtime.

## Authority

This crate is authoritative for Python install surfaces and bridge behavior,
but it must preserve the command semantics already owned by `bijux-cli`.

## Invariants

- Python entrypoints must preserve native runtime semantics instead of redefining them
- bridge conversions must keep machine-readable outputs compatible with `bijux-cli`
- packaging metadata must point users back to the primary CLI runtime contract
- maintainer-only workflows and DAG-specific semantics remain outside this crate boundary

## Related tests

- `crates/bijux-cli-python/tests/runtime_entrypoint_unity.rs`
- `crates/bijux-cli-python/tests/python_packaging_ownership.rs`
- `crates/bijux-cli-python/tests/bridge_bindings.rs`
- `crates/bijux-cli-python/tests/python/test_packaging_contracts.py`
- `crates/bijux-cli-python/tests/python/test_runtime_parity.py`

## Related schemas

None. This crate consumes CLI runtime output and package metadata contracts
through `bijux-cli` rather than defining an independent schema surface.

## Versioning and change policy

Python launcher behavior, bridge conversion semantics, and packaging ownership
must remain compatible with the native `bijux-cli` runtime. Any incompatible
change requires updating this document and the linked Rust and Python parity
tests in the same change.
