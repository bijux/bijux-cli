# As-Underscore Import Policy

## Purpose

`use ... as _;` is allowed only where it is the clearest way to satisfy dependency-touch or trait-visibility boundaries.

## Allowed cases

- Crate integration tests under `crates/*/tests/*.rs` for dependency-touch imports required by strict `unused-crate-dependencies` hygiene.
- Bench targets under `crates/*/benches/*.rs` for dependency-touch imports.
- Crate root entrypoints (`src/lib.rs`, `src/main.rs`, `src/bin/*.rs`) when dependency-touch imports are required by target-level dependency accounting.

## Disallowed cases

- Internal module implementation files outside test/bench and crate root entrypoints.
- Decorative `as _` aliases where a normal explicit import is clearer.

## Enforcement

- Policy source: `configs/policy/as_underscore_import_policy.json`
- Contract test: `crates/bijux-dev-dag/tests/as_underscore_import_contracts.rs`
- Audit report: `docs/reports/foundation/as_underscore_import_audit.md`

## Review rule

Every new `use ... as _;` must fit one allowed path class or an explicit exception with a written reason.
