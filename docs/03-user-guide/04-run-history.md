# Run History

Describe how run history should be used to track behavior over time and support replay/diff-driven diagnosis.

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

Run lookup patterns:
- exact lookup by run ID for targeted diagnosis.
- latest-N lookup for operational trend review.
- baseline lookup (known-good run) for replay/diff anchor.

Indexing expectations:
- stable ordering by creation or start time.
- fast direct access by run ID.
- preserved linkage to graph and artifact references.

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

# Lookup baseline then compare
bijux-dag inspect run --run-id RUN_20260309_007

# Compare latest failing run against baseline
bijux-dag diff run --left RUN_20260309_007 --right RUN_20260309_010
```

```text
Run history inspection example:
- RUN_007: succeeded (baseline)
- RUN_008: succeeded
- RUN_009: failed (node transform timeout)
- RUN_010: failed (node transform timeout)
Interpretation:
- recurring failure at same node suggests deterministic issue, not random flake.
```

```text
Run comparison example:
left: RUN_007 (known good)
right: RUN_010 (failing)
diff scope:
- graph: equivalent
- run: drift at node transform
- artifact: missing output from transform
```

## Guarantees
- Run history is treated as an active operational surface.
- Guidance here aligns with replay and diff workflows.
- Lookup, indexing, inspection, and comparison paths are explicit.

## Limitations
- This guide does not define persistence engine internals.
- Retention and archival policy is deployment-dependent.

## Related
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/06-diff.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
- `docs/06-specification/02-run-model.md`
