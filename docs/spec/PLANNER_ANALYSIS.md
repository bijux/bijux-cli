# Planner analysis and optimization contracts

## Planner phase model

Planner execution is represented as explicit phases:

1. normalize
2. validate
3. bind
4. optimize
5. schedule-ready transform

## Selection, replay, and backfill planning

- Node annotations capture why a node is selected, deferred, skipped, or replayed.
- Replay plans distinguish execute and skip actions.
- Partial-run closure expansion is deterministic.
- Backfill plans include explicit window boundaries and partition keys.

## Resource and placement intelligence

- Planner estimates aggregate CPU and memory requirements from node contracts.
- Priority inheritance is derived from graph and node policy hints.
- Locality and queue placement hints are emitted as plan annotations.

## Compatibility and guardrails

- Planner validates backend capability compatibility before execution.
- Impossible runs are rejected at plan time (for example invalid resource contracts).
- Optimizer rules may only alter behavior when guardrails explicitly permit semantic optimization.

## Fingerprints, diffs, and explainability

- Plan fingerprints are stable identities for equivalent plan outputs.
- Plan diffs capture order/filter/annotation changes.
- Explain-plan output summarizes phases, annotations, and optimization notes.

## Benchmark fixture set

- `evidence/perf/fixtures/planner_large_fanout.json`
- `evidence/perf/fixtures/planner_deep_chain.json`
- `evidence/perf/fixtures/planner_mixed_resources.json`
