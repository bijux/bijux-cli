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
