# Battle workflow contract

## Scope

Battle workflows are executable stress scenarios for runtime behavior under realistic pressure.

## Scenario catalog

Fixtures live in `crates/bijux-dag-runtime/tests/fixtures/battle_workflows` and are validated by `battle_workflow_harness_contracts.rs`.

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
