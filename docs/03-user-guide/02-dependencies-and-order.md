# Dependencies And Order

Explain how dependency edges determine execution order and operational behavior.

Dependency modeling is core to correctness and performance in DAG execution.

## Explanation
Dependency edge semantics:
- edge `A -> B` means `B` cannot start before `A` completes successfully.

Execution order model:
- nodes with satisfied prerequisites become schedulable
- independent nodes may run concurrently
- constrained nodes wait for prerequisites

Declaration order vs execution order:
- declaration order is author convenience and review readability.
- execution order is determined by dependency graph, not file position.
- if two nodes have no dependency relation, runtime may execute them in either order.

Fan-in, fan-out, and partial-order patterns:
- fan-in: many prerequisites feed one node (for example merge/join).
- fan-out: one prerequisite feeds many independent downstream nodes.
- partial-order: only constrained subsets are ordered; unrelated nodes stay concurrent.

Topological ordering:
- scheduler computes an order consistent with all directed edges.
- many valid topological orders can exist for the same graph.
- deterministic scheduling means classification and dependency-correctness remain stable, even when parallel execution affects wall-clock interleaving.

Deterministic scheduling boundaries:
- dependency constraints are strict and non-negotiable.
- concurrency is allowed only for nodes with satisfied prerequisites and no unsatisfied inbound edges.
- if prerequisites fail, dependents stay blocked/skipped according to policy.

Cycle detection:
- cycle presence makes topological ordering impossible.
- cycle detection is a mandatory graph validation gate before execution.
- execution must fail fast on cycles rather than attempting partial unordered runs.

Dependency modeling rules:
- include only true data/control prerequisites
- avoid chain-shaped graphs when independent work is possible
- avoid implicit dependencies hidden in scripts

Common modeling errors:
- missing required prerequisite edge
- unnecessary edge reducing concurrency
- dependency on node name that does not exist
- assuming declaration order forces runtime order
- fan-out omitted by hidden script dependency

## Examples
```json
{
  "nodes": [
    {"id": "build-a", "depends_on": []},
    {"id": "build-b", "depends_on": []},
    {"id": "merge", "depends_on": ["build-a", "build-b"]}
  ]
}
```

```text
Execution implication:
- build-a and build-b can run in parallel
- merge waits for both
```

```text
Fan-out example:
prepare -> lint
prepare -> test
prepare -> package
All downstream nodes may run once prepare succeeds.
```

```text
Topological order examples:
- valid order 1: build-a, build-b, merge
- valid order 2: build-b, build-a, merge
Both are dependency-correct.
```

```text
Cycle example:
A depends on B, B depends on C, C depends on A
-> validation error: cycle detected
```

```text
Runtime symptom of dependency mistakes:
- missing true edge can cause downstream node to read unavailable input and fail at runtime.
- unnecessary edge can serialize work and increase duration without correctness benefit.
```

## Guarantees
- Dependency edge behavior is described as strict prerequisite logic.
- Guidance here is compatible with scheduler and run semantics docs.
- Includes explicit topological ordering, parallelism, and cycle constraints.

## Limitations
- This document does not specify scheduler internals or optimization strategy.
- It does not cover advanced distributed scheduling controls.

## Related
- `docs/03-user-guide/01-authoring-dags.md`
- `docs/05-system-architecture/04-scheduler.md`
- `docs/06-specification/01-dag-model.md`
- `docs/06-specification/08-diff-semantics.md`

## Fan-in, fan-out, and partial order

Dependency structure defines a partial order, not a single global sequence.

- `fan-out`: one producer node feeds multiple downstream nodes that can run in parallel after producer success.
- `fan-in`: one consumer node waits for multiple upstream nodes and executes only when all required parents succeed.
- `partial order`: unrelated subgraphs are intentionally unordered and may execute concurrently.

Example shape:

- `extract` -> `clean_a`
- `extract` -> `clean_b`
- `clean_a` + `clean_b` -> `join`

`clean_a` and `clean_b` are unordered relative to each other, but both must precede `join`.

## Declaration order versus execution order

The order nodes appear in a file is documentation convenience. Execution order is computed from dependency edges plus scheduler rules. Reordering declarations without changing edges should not change semantic ordering constraints.

## How dependency mistakes appear at runtime

Typical symptoms map directly to dependency bugs:

- Missing edge: consumer runs too early and fails with missing input artifact.
- Extra edge: graph becomes unnecessarily serialized and slower.
- Wrong parent edge: consumer sees valid but incorrect upstream data.
- Hidden cycle through generated intermediates: validation rejects graph or scheduler cannot produce a legal order.

When these appear, inspect edge definitions first, then validate graph identity impact before retrying runs.
