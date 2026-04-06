# What Is Bijux Dag

Most pipeline systems fail at the same moment: output changed, a release is blocked, and nobody can prove whether the cause was graph definition drift, runtime behavior drift, or output drift. Bijux-dag exists to make that question mechanically answerable.

## The problem bijux-dag is built to solve

Typical workflow stacks are good at triggering jobs and collecting logs, but weak at producing comparable execution evidence across time. Teams then debug by inference, not by contract:

- they know a run failed, but not whether the definition changed;
- they know an artifact changed, but not whether it was expected;
- they can rerun, but cannot classify equivalence versus bounded divergence reliably.

Bijux-dag narrows scope to fix that exact failure mode: deterministic graph/run/artifact identity, explicit run evidence, replay classification, and diff classification.

## What bijux-dag is

Bijux-dag is a DAG execution and evidence system focused on deterministic operational control. It is not an orchestration platform trying to own every scheduling topology and integration surface.

Design emphasis:

- explicit graph semantics over implicit runtime behavior,
- identity-backed attribution over best-effort log narratives,
- replay/diff as normal operational controls rather than optional diagnostics.

## “Git for computation graphs”: useful mapping and hard limits

| Git concept | Bijux-dag analogue | Why this helps |
| --- | --- | --- |
| commit identity | graph/run/artifact identity | stable reference for comparison |
| diff | graph/run/artifact diff | scoped change classification |
| checkout/verify workflows | replay workflows | evidence-based equivalence checks |

Where the analogy stops: bijux-dag does not try to mirror Git’s storage model, branching model, or command semantics. The phrase is an intuition shortcut, not a compatibility claim.

## Full lifecycle walkthrough

```text
graph definition -> run execution -> artifact lineage -> replay classification -> diff classification -> release decision
```

Worked operator path:

```bash
bijux-dag run --dag ./pipelines/orders.dag.json
bijux-dag inspect run --run-id RUN_20260309_220
bijux-dag inspect artifact --artifact-id ART_20260309_902
bijux-dag replay --run-id RUN_20260309_220
bijux-dag diff run --left RUN_20260309_204 --right RUN_20260309_220
```

Interpretation flow:

1. `run` creates attributable execution evidence.
2. `inspect` confirms failure/success location and output lineage.
3. `replay` answers whether behavior reoccurs under replay semantics.
4. `diff` classifies what changed and where.

## Guarantees you can test

- A run can be referenced by run identity and inspected without re-executing the graph.
- Replay results produce explicit classifications, not only free-form logs.
- Diff results are scope-specific (`graph`, `run`, `artifact`) and do not collapse all change into one bucket.

## Non-goals and limits

- Bijux-dag does not guarantee universal backend equivalence.
- Deterministic classification does not imply identical wall-clock timing.
- Successful transport of bundles does not, by itself, prove portability equivalence.

## Common wrong assumption

“Deterministic” means “everything is byte-for-byte identical everywhere.” In bijux-dag, determinism claims are bounded by declared identity policy and backend capability envelope.

## Next reading

- Mission and scope constraints: [Mission](../01-introduction/02-mission.md)
- Tradeoff rules that drive design: [Design Principles](../01-introduction/03-design-principles.md)
- Object model and relationships: [Core Concepts](../01-introduction/04-core-concepts.md)
- Normative DAG contract: [DAG Model Specification](../06-specification/01-dag-model.md)
