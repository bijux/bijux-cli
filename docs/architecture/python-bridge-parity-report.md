# Python Bridge Parity Report

Date: 2026-03-09

## Coverage summary

Implemented and tested bridge command bindings:

- `version`
- `doctor`
- `status`
- `cli status`
- `plugins list`

Other covered bridge behavior:

- error envelope/error-kind mapping surface
- config precedence resolution API
- plugin namespace rejection mapping
- REPL bootstrap help path (`repl --help`)
- schema export helper API
- bridge output parity against direct `run_app` for covered commands

## Evidence

Test file:

- `crates/bijux-cli-python/tests/bridge_bindings.rs`

## Known gaps

1. Python exception class mapping currently validated by error-kind tags, not concrete Python exception subclasses.
2. Bridge parity for more command families (`config`, `history`, `memory`) is not yet covered.
3. End-to-end validation against a packaged Python wheel runtime is pending.

## Next crate-local steps

1. Add typed Python exception wrappers around bridge `error_kind` tags.
2. Expand covered command set to include config and diagnostics flows.
3. Add packaged-runtime parity job in CI.
