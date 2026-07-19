# bijux-dag-cli Contracts

Responsibility: Thin process entrypoint that delegates to app command surfaces.

## Scope
`bijux-dag-cli` is a thin binary wiring crate. It owns argument entrypoint wiring and final error-to-exit mapping only.

## Authority
This crate is authoritative for process entrypoint behavior and top-level CLI boot sequence.

## Invariants
- No domain business logic.
- No runtime internals.
- Delegates command execution to `bijux-dag-app`.

## Allowed changes
- Wiring and startup flow improvements.
- Error mapping refinements that preserve published error contract behavior.

## Related tests
- `crates/bijux-dag-cli/tests/smoke_pipeline.rs`
- `crates/bijux-dag-app/tests/cli_contract.rs`

## Related schemas
None. This crate consumes schemas indirectly through app/runtime crates.

## Versioning and change policy
Entrypoint changes must preserve published command tree and exit behavior unless explicitly documented as breaking.
