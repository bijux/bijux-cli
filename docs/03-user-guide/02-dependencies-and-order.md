# Dependencies And Order

## Purpose
Explain how dependency edges determine execution order and operational behavior.

## Context
Dependency modeling is core to correctness and performance in DAG execution.

## Explanation
Dependency edge semantics:
- edge `A -> B` means `B` cannot start before `A` completes successfully.

Execution order model:
- nodes with satisfied prerequisites become schedulable
- independent nodes may run concurrently
- constrained nodes wait for prerequisites

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

## Guarantees
- Dependency edge behavior is described as strict prerequisite logic.
- Guidance here is compatible with scheduler and run semantics docs.

## Limitations
- This document does not specify scheduler internals or optimization strategy.
- It does not cover advanced distributed scheduling controls.

## Related
- `docs/03-user-guide/01-authoring-dags.md`
- `docs/05-system-architecture/04-scheduler.md`
- `docs/06-specification/01-dag-model.md`
- `docs/06-specification/08-diff-semantics.md`
