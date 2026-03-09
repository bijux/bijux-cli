# Execution Engine

## Purpose
Describe how the execution engine runs nodes, handles outcomes, and coordinates with scheduling.

## Context
The execution engine is the runtime core that turns DAG plans into concrete run behavior.

## Explanation
Execution engine responsibilities:
- consume scheduler-ready node work units
- execute node commands through adapter/runtime boundaries
- capture success/failure outcomes
- emit state transitions and operational evidence

Engine behavior model:
- node execution follows dependency readiness
- failures are surfaced with diagnosable state
- outputs are handed to artifact persistence surfaces

Engine boundary rules:
- engine executes; scheduler orders
- engine records outcomes; history/index surfaces present trends

Deliberate non-goals:
- engine is not a policy layer for cross-backend equivalence decisions
- engine does not redefine adapter capability boundaries
- engine does not treat wall-clock identity as determinism

## Examples
```text
Engine cycle:
receive schedulable node -> execute -> record result -> notify scheduler/state
```

## Guarantees
- Engine responsibilities are separated from scheduler responsibilities.
- Outcome handling is described as explicit stateful behavior.

## Limitations
- This page does not define all runtime state-machine internals.
- Adapter-specific execution details are covered in adapter architecture docs.
- Performance tuning heuristics are outside this architecture baseline.

## Related
- `docs/05-system-architecture/04-scheduler.md`
- `docs/05-system-architecture/05-adapters.md`
- `docs/05-system-architecture/07-run-directory.md`
- `docs/06-specification/02-run-model.md`
