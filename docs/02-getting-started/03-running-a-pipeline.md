# Running A Pipeline

## Purpose
Show how to execute a DAG, inspect outcomes, and perform first replay/diff checks.

## Context
This document turns authored DAG definitions into operational runs.

## Explanation
Pipeline execution flow:
1. Submit DAG for run.
2. Observe run lifecycle state.
3. Inspect outputs/artifacts.
4. Replay for behavioral confidence.
5. Diff runs when outcomes diverge.

Run lifecycle summary:
- created
- executing
- completed or failed

Run directory layout is created per execution and contains runtime evidence needed for inspect/replay/diff operations.

Replay usage:
- Use replay when verifying stability across repeated execution.
- Replay is especially useful after environment or dependency changes.

Diff usage:
- Use run diff to compare outcomes between two run IDs.
- Diff helps classify whether change is expected or suspicious.

## Examples
```bash
# Run
bijux-dag run --dag ./examples/first.dag.json

# Inspect run details
bijux-dag inspect run --run-id RUN_20260309_001

# Replay run context
bijux-dag replay --run-id RUN_20260309_001

# Diff two runs
bijux-dag diff run --left RUN_20260309_001 --right RUN_20260309_002
```

## Guarantees
- Basic run, inspect, replay, and diff flow is documented as one coherent path.
- Lifecycle states are described in the same order users observe operationally.

## Limitations
- Advanced lane/backend behavior is not covered here.
- Field-level inspect and diff semantics are defined in specification docs.

## Related
- `docs/02-getting-started/04-understanding-runs.md`
- `docs/03-user-guide/04-run-history.md`
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/06-diff.md`
