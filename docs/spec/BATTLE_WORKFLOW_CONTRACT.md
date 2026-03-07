# Battle workflow contract

## Scope

Battle workflows are executable stress scenarios for runtime behavior under realistic pressure.

## Scenario catalog

Fixtures live in `evidence/battle/workflows/runtime` and are validated by `battle_workflow_harness_contracts.rs`.

## Required scenarios

- medium workflow
- failure-heavy workflow
- artifact-heavy workflow
- cache invalidation workflow
- replay divergence workflow
- scheduler fairness workflow
- import/export workflow
- corruption workflow
- operator inspection workflow
- large dag workflow
- resource contention workflow
- multi-root workflow
- branch/join workflow
- retry storm workflow
- timeout workflow
- version compatibility workflow
- malformed run-dir workflow
- ugly realistic dag workflow
- policy violation workflow
- secret leakage workflow
- operator debugging workflow

## Fixture requirements

Each scenario fixture must include:

- `scenario`
- `graph`
- `nodes`
- `focus`
- `expectations`

## Non-negotiable properties

- State-machine conformance is mandatory evidence for battle workflows.
- Node and run transitions must satisfy the state-machine contract and invariant IDs.
- Replay battle scenarios must include mandatory replay proof assertions and semantic diff evidence.

## Ownership metadata

- Scenario ownership and retention metadata live in `evidence/battle/metadata.json`.
- Required fields per scenario:
  - `grade`
  - `why_exists`
  - `delete_review`

## Trust property mapping

- Trust properties and scenario coverage are normative in `configs/policy/battle_trust_properties.json`.
- Every battle scenario must map to one or more trust properties.
- Orphan mappings and orphan metadata entries are rejected by battle drift checks.

## Verification gates

- `cargo nextest run` executes `battle_workflow_harness_contracts`.
- `make test-all` must keep battle checks green.
- `bijux-dev-dag foundation` must include `battle-suite-mandatory` in repo governance checks.
