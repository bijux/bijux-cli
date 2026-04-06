# Scheduler

Scheduler guarantees dependency-correct progression while balancing determinism, fairness, and bounded concurrency.

## Scheduler goals

- deterministic eligibility from the same dependency state,
- fair progress for concurrently-ready nodes,
- bounded concurrency under runtime limits,
- explicit terminal handling for blocked branches.

## Non-goals

Scheduler does not decide:

- backend invocation mechanics,
- artifact storage policy,
- semantic meaning of node payload contents,
- portability acceptance decisions.

## Readiness frontier examples

Example A:

- edges: `extract -> clean_a`, `extract -> clean_b`, `clean_a + clean_b -> join`

Frontier evolution:

1. ready: `extract`
2. after `extract` success: `clean_a`, `clean_b`
3. after both succeed: `join`

Example B (failure path):

- edges: `prepare -> train -> publish`

If `train` fails, frontier never includes `publish`; branch remains blocked by dependency failure.

## Determinism note

Deterministic scheduling means stable eligibility semantics, not identical wall-clock interleaving of parallel nodes.

## Next reading

- Execution handoff behavior: [Execution Engine](../05-system-architecture/03-execution-engine.md)
- Authoring dependency consequences: [Dependencies And Order](../03-user-guide/02-dependencies-and-order.md)
