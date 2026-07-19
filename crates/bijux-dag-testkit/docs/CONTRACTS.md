# bijux-dag-testkit Contracts

Responsibility: Shared deterministic test fixtures, builders, and assertion helpers for workspace crates.

## Scope
`bijux-dag-testkit` provides reusable test-only fixtures, builders, matchers, and harness utilities shared across crates and top-level suites.
It is maintained as a repository-internal support crate rather than a public
release surface.

## Authority
This crate is the single source of truth for shared DAG fixture builders and run-artifact test assertions.

## Invariants
- Production crates must not depend on this crate at runtime.
- Public crates must not require this crate to package or run.
- Utilities remain deterministic and side-effect-bounded for tests.
- Shared fixtures are canonicalized and reusable across test families.

## Allowed changes
- Add or refine fixture builders and assertion helpers.
- Introduce test harness utilities for new test families.

## Related tests
- `crates/bijux-dag-runtime/src/internal/testing/test_support.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`

## Related schemas
- `configs/dag/schema/dag.schema.json`
- `configs/dag/schema/run_manifest.schema.json`
- `configs/dag/schema/node_trace.schema.json`

## Versioning and change policy
APIs are internal to repository tests; compatibility is maintained across workspace crates, not across external semver users.
