# FIXTURE TEST AND VALIDATION GOVERNANCE

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/CANONICAL_FIXTURE_STRATEGY_POLICY.md
# Canonical Fixture Strategy Policy

## Objective

Reduce fixture sprawl by preferring canonical reusable fixtures and explicit governance tags.

## Rules

1. Fixture families must be governed by owner, suite, and purpose.
2. Fixtures must use one of: `canonical`, `stress`, `corrupt`, `smoke`, `legacy`.
3. Smoke defaults may only use `canonical` or `smoke` tags.
4. Orphan and duplicate fixtures are cleanup candidates and must be tracked.

## Enforcement

- `configs/policy/fixture_family_governance.json`
- `configs/suites/fixture_contraction_verification.json`
- `crates/bijux-dev-dag/tests/fixture_canonicalization_contracts.rs`

## SOURCE: docs/spec/FIXTURE_TOOLING_GOVERNANCE_CONTRACT.md
# Fixture Tooling Governance Contract

## Purpose

Define required fixture-tooling guarantees so generated fixtures remain deterministic, portable, and continuously verifiable.

## Required fixture tooling capabilities

- test fixture generation utility coverage for graph, run, artifact, replay, diff, and bundle families
- corpus generation support for deterministic, fuzz, and benchmark scenarios
- machine-readable fixture validation surfaces and schema checks
- fixture duplication detection and lifecycle cleanup governance
- governance reports that explain coverage and ownership of fixture families

## Required verification surfaces

- fixture schema validation tests
- fixture determinism tests
- fixture portability tests
- fixture governance completion contracts in `bijux-dev-dag`

## Required governance artifacts

- fixture tooling regression corpus
- fixture tooling governance suite definition
- fixture tooling coverage report
- fixture duplication detection report
- fixture cleanup automation report
- fixture lifecycle governance report

## SOURCE: docs/spec/SPEC_TO_CODE_AND_TEST_OWNERSHIP.md
# Spec to code and test ownership

## Scope

This mapping binds normative specs to owning code paths and owning test suites.

| spec | owning code path | owning test suite |
| --- | --- | --- |
| `docs/spec/EXECUTION_SEMANTICS_CONTRACT.md` | `crates/bijux-dag-runtime/src/runtime_core` | `crates/bijux-dag-runtime/tests/runtime_semantics_contracts.rs` |
| `docs/spec/SCHEDULER_CONTRACT.md` | `crates/bijux-dag-runtime/src/runtime_core/scheduler.rs` | `crates/bijux-dag-runtime/tests/runtime_scheduler_contracts.rs` |
| `docs/spec/STATE_MACHINE_CONTRACT.md` | `crates/bijux-dag-runtime/src/state_machine` | `crates/bijux-dag-runtime/tests/state_machine_transitions.rs` |
| `docs/spec/CACHE_CONTRACT.md` | `crates/bijux-dag-runtime/src/cache` | `crates/bijux-dag-runtime/tests/runtime_cache_contracts.rs` |
| `docs/spec/REPLAY_CONTRACT.md` | `crates/bijux-dag-runtime/src/replay` | `crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs` |
| `docs/spec/IMPORT_EXPORT_CONTRACT.md` | `crates/bijux-dag-app/src/import_export` | `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs` |
| `docs/spec/OPERATOR_UX_CONTRACT.md` | `crates/bijux-dag-app/src/inspect` | `crates/bijux-dag-app/tests/operator_ux_contract.rs` |
| `docs/spec/CRATE_BOUNDARY_CONTRACT.md` | `crates/bijux-dev-dag/src/commands` | `crates/bijux-dev-dag/tests/crate_taxonomy_guardrails.rs` |
| `docs/spec/EVIDENCE_MODEL.md` | `crates/bijux-dev-dag/src/commands` | `crates/bijux-dev-dag/tests/evidence_governance_contract.rs` |
| `docs/spec/TEST_TRUST_LEDGER.md` | `configs/policy/test_trust_ledger.json` | `crates/bijux-dev-dag/tests/test_trust_cleanup_contracts.rs` |

## Rule

Every normative spec in `docs/spec/` must remain mapped to one code path and one test suite.

## SOURCE: docs/spec/TEST_PHILOSOPHY.md
# Test philosophy

## Goal

Tests are evidence for normative behavior, not implementation snapshots.

## Principles

- Prefer contract-level assertions over structural assertions.
- Prefer deterministic assertions over timing-sensitive assumptions.
- Prefer explicit failure-path checks for every normative success path.
- Keep one primary owner test per contract surface.
- Keep fixtures executable and versioned with the test that uses them.

## Runtime trust strategy

- semantic contract tests define expected behavior
- adversarial tests prevent unsafe assumptions
- failure-path tests enforce explicit error classes
- replay tests enforce deterministic equivalence expectations
- scheduler tests enforce deterministic ordering and fairness

## Promotion policy

A test is promoted to critical when it guards deterministic execution, state transition safety, cache proof correctness, artifact integrity, policy safety, or replay equivalence.

## SOURCE: docs/spec/TEST_STRATEGY.md
# Test strategy

## Testing pyramid

1. unit
2. module
3. contract
4. integration
5. end-to-end
6. replay
7. fault-injection
8. performance
9. compatibility

## Crate ownership and forbidden test types

- `bijux-dag-core`
  - owns: unit, module, contract, compatibility fixtures
  - forbidden: end-to-end, process-spawn runtime execution
- `bijux-dag-artifacts`
  - owns: contract, integration, corruption/fault artifact tests
  - forbidden: scheduler behavior tests
- `bijux-dag-runtime`
  - owns: state-machine, execution contract, cache, replay, fault tests
  - forbidden: direct CLI UX snapshot tests
- `bijux-dag-app`
  - owns: command integration and error-path tests
  - forbidden: runtime internal state transition tests
- `bijux-dag-cli`
  - owns: binary wiring and exit-code mapping tests
  - forbidden: runtime planning/execution internals
- `bijux-dev-dag`
  - owns: governance, policy, contract, release discipline checks
  - forbidden: product runtime behavior tests

## Universal rules

- Only e2e tests may shell out to production binaries.
- Every public command requires one integration test and one error-path test.
- Every schema requires positive and negative fixtures.
- Runtime state transitions require explicit transition coverage.
- Cache behavior requires `off`, `read`, and `readwrite` mode coverage.

## SOURCE: docs/spec/VALIDATION_RULES.md
# Validation Rules

Validation rule registry source: `crates/bijux-dag-core/src/validate.rs`.

## Validation domains
- `Schema`: structural and shape constraints.
- `Semantic`: behavior and meaning constraints.
- `Topology`: graph connectivity and ordering constraints.

## Error Codes
- `E1001` Duplicate node id
- `E1002` Dangling node reference
- `E1003` Dangling port reference
- `E1004` Cycle detected
- `E1005` JSON parse error / unknown fields
- `E1006` Invalid spec version
- `E1007` Illegal node id characters
- `E1008` Output collision
- `E1009` Missing effects declaration
- `E1010` Env allowlist without env effect
- `E1011` Retry disallowed for nondeterministic effects
- `E1013` Effect denied by policy (network/env/clock)
- `E1020` Unknown graph input reference
- `E1021` Unknown node output reference
- `E1022` Forward node output reference
- `E1023` Missing container spec for container node
- `E1024` Invalid container spec
- `E1025` Invalid output file path
- `E1026` Illegal tag name
- `E1027` Illegal graph name

## Warning Codes
- `W2001` Unreachable node
- `W2002` Orphan node

## Rules
1. Node ids must be unique. (`E1001`)
2. Edge node references must exist. (`E1002`)
3. Edge port references must exist on their nodes. (`E1003`)
4. The graph must be acyclic. (`E1004`)
5. JSON must be strict with no unknown fields. (`E1005`)
6. DAG spec version must be known. (`E1006`)
7. Node ids must match `[a-zA-Z0-9_-]+`. (`E1007`)
8. Output names must be unique across nodes. (`E1008`)
9. Shell nodes must declare effects and include filesystem. (`E1009`)
10. env_allowlist requires env effect. (`E1010`)
11. Retry with clock/network requires random_seed or nondeterminism_allowed. (`E1011`)
12. Parameter references must be valid graph inputs or node outputs. (`E1020`, `E1021`, `E1022`)
13. Effect denied by policy when `--deny-network`, `--deny-env`, or `--deny-clock` used. (`E1013`)
14. Graph input ref must exist. (`E1020`)
15. Node output ref must exist. (`E1021`)
16. Node output ref must not point to downstream node. (`E1022`)
17. Container nodes must include a container spec. (`E1023`)
18. Container spec must be valid (engine and argv). (`E1024`)
19. Output file paths must be relative and not contain `..`. (`E1025`)
20. Tag names must match `[a-zA-Z0-9_-]+`. (`E1026`)
21. Graph name must match `[a-zA-Z0-9_-]+`. (`E1027`)
22. Nodes not reachable from any root emit a warning. (`W2001`)
23. Nodes with no edges emit a warning. (`W2002`)
