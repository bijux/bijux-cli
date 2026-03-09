# Run History

## Purpose
Describe how run history should be used to track behavior over time and support replay/diff-driven diagnosis.

## Context
Run history is the operational bridge between one-off execution and long-term workflow reliability.

## Explanation
Run history captures ordered run records for a DAG or operating scope.

What to inspect first:
- run ID sequence
- terminal status trend
- recurring failure points
- duration and stability drift

Run storage model (user-level view):
- each run persists execution context and outcome metadata
- historical runs remain reference points for comparison
- stable run identity enables longitudinal analysis

Run indexing model (user-level view):
- run records are retrievable by run ID
- ordering supports chronological comparison and triage
- recent and baseline runs should be easy to locate for replay and diff

Practical usage pattern:
1. identify the latest failing run
2. pick a known-good baseline run
3. replay baseline or failing run as needed
4. run diff to classify divergence

## Examples
```bash
# List recent runs
bijux-dag run history --limit 20

# Inspect a specific run
bijux-dag inspect run --run-id RUN_20260309_010

# Compare latest failing run against baseline
bijux-dag diff run --left RUN_20260309_007 --right RUN_20260309_010
```

## Guarantees
- Run history is treated as an active operational surface.
- Guidance here aligns with replay and diff workflows.

## Limitations
- This guide does not define persistence engine internals.
- Retention and archival policy is deployment-dependent.

## Related
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/06-diff.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
- `docs/06-specification/02-run-model.md`
