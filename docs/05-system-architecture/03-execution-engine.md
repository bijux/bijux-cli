# Execution Engine

The execution engine turns scheduler-ready nodes into recorded outcomes and artifact evidence.

## Stateful execution sequence

For each ready node, the engine follows this sequence:

1. accept ready node from scheduler frontier,
2. bind run context and dependency evidence,
3. invoke adapter execution,
4. normalize backend result to canonical outcome,
5. persist node outcome and artifact references,
6. return terminal node status to scheduler.

This is the core path from intent to auditable evidence.

## Responsibility split

- scheduler decides eligibility order,
- engine executes eligible work and records outcomes,
- adapter translates runtime intent to backend-native invocation.

Engine must not reorder dependency semantics; scheduler must not interpret backend-native exit details.

## Why normalized outcomes exist

Normalized outcomes keep comparison logic stable across backends. Without normalization, replay/diff would depend on backend-specific error shapes and lose portability of interpretation.

Canonical outcome envelopes also improve incident triage because reason classes remain comparable across runs.

## Next reading

- Eligibility and readiness rules: [Scheduler](../05-system-architecture/04-scheduler.md)
- Backend translation contract: [Adapters](../05-system-architecture/05-adapters.md)
- Run evidence contract: [Run Model Specification](../06-specification/02-run-model.md)
