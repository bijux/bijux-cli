# Dependencies And Order

Dependencies define legal execution order. Declaration order is only authoring layout.

## Partial order comes first

A DAG defines a partial order:

- if `A -> B`, then B must wait for A;
- if two nodes are unrelated by dependency path, they may run in either order or concurrently.

This is why deterministic scheduling does not imply one fixed wall-clock sequence.

## Common patterns

Fan-in:

- `clean_a` + `clean_b` -> `join`

Fan-out:

- `normalize` -> `report`
- `normalize` -> `quality_checks`

Linear chain:

- `extract` -> `transform` -> `publish`

Use fan-in/out intentionally. Overusing chains reduces concurrency without improving correctness.

## How dependency mistakes surface

Planning-time symptoms:

- cycle detected,
- unknown dependency target,
- unschedulable graph.

Runtime symptoms:

- consumer runs too early due to missing edge,
- graph becomes slow due to unnecessary edge,
- downstream node consumes wrong parent output due to miswired edge.

## Determinism without fixed clocks

Scheduler determinism means dependency-correct eligibility and stable classification semantics, not identical wall-clock interleaving of parallel-ready nodes.

## Next reading

- Practical authoring choices: [Authoring Dags](../03-user-guide/01-authoring-dags.md)
- Scheduler behavior details: [Scheduler Architecture](../05-system-architecture/04-scheduler.md)
