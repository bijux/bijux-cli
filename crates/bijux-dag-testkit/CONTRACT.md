# bijux-dag-testkit Contract

## Scope
`bijux-dag-testkit` provides reusable test-only fixtures, builders, matchers, and harness utilities shared across crates and top-level suites.

## Authority
This crate is the single source of truth for shared DAG fixture builders and run-artifact test assertions.

## Invariants
- Production crates must not depend on this crate at runtime.
- Utilities remain deterministic and side-effect-bounded for tests.
- Shared fixtures are canonicalized and reusable across test families.

## Allowed changes
- Add or refine fixture builders and assertion helpers.
- Introduce test harness utilities for new test families.

## Related tests
- `crates/bijux-dag-runtime/src/test_support.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`

## Related schemas
- `configs/schema/dag.schema.json`
- `configs/schema/run-manifest.schema.json`
- `configs/schema/node-trace.schema.json`

## Versioning and change policy
APIs are internal to repository tests; compatibility is maintained across workspace crates, not across external semver users.
