# Tests Contract

## Scope
`tests/` contains top-level test suites that validate product-level behavior beyond per-crate unit/module tests.

## Authority
This directory is authoritative for top-level test orchestration and execution code.
Scenario truth is evidence-owned; tests consume scenario assets from `evidence/`.

## Invariants
- Test family taxonomy follows `unit_`, `contract_`, `integration_`, `e2e_`, `perf_`, `compat_`, `fault_`.
- E2E-only tests may shell out to production binaries.
- Non-E2E tests use crate-level entrypoints or testkit utilities.

## Allowed changes
- Add new suites under documented family structure.
- Expand coverage maps and debt ledgers with explicit ownership.

## Related tests
- `tests/e2e/*`
- `evidence/fault/*`
- `crates/bijux-dev-dag/src/commands/mod.rs` taxonomy guards

## Related schemas
- `configs/policy/test_taxonomy.json`

## Versioning and change policy
Test taxonomy changes require simultaneous updates to control-plane guards and test strategy docs.
