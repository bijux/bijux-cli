# Run History

## Purpose
Describe how run history is used to track behavior over time and support troubleshooting.

## Context
Run history is a core operational surface for regression detection and replay/diff workflows.

## Explanation
Run history captures ordered run records for a DAG or operational scope.

What users should read from run history first:
- run ID sequence
- status trends
- failure concentration points
- time-based behavior drift

Run history usage patterns:
- compare current run against last known good run
- identify recurring failure node families
- select replay baseline candidates

Run indexing concepts:
- history entries are retrievable by run identity
- ordering should support chronological reasoning

## Examples
```bash
# Example run history command surface
bijux-dag run history --limit 20

# Compare latest two relevant runs
bijux-dag diff run --left RUN_20260309_010 --right RUN_20260309_011
```

## Guarantees
- Run history is treated as an operational diagnosis surface, not a passive log dump.
- Guidance aligns with run model and replay/diff user flows.

## Limitations
- This guide does not define persistence engine internals.
- Retention policy details are environment-dependent.

## Related
- `docs/02-getting-started/04-understanding-runs.md`
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/06-diff.md`
- `docs/06-specification/02-run-model.md`
