# Inspect And Debug

Use inspect as your first evidence source, then replay and diff to validate and classify what you found.

## Authoritative versus derived inspect surfaces

Authoritative surfaces:

- run terminal status and node outcomes,
- artifact identity and lineage links,
- recorded failure reason classes.

Derived surfaces:

- rollup summaries,
- trend dashboards,
- cached diagnostic shortcuts.

If authoritative and derived views disagree, trust authoritative run/artifact records.

## Worked failure investigation

Scenario:

- run `RUN_20260309_211` failed,
- expected artifact `out/orders_normalized.parquet` missing.

Workflow:

```bash
bijux-dag inspect run --run-id RUN_20260309_211
bijux-dag inspect artifact --run-id RUN_20260309_211
bijux-dag replay --run-id RUN_20260309_211
bijux-dag diff run --left RUN_20260309_204 --right RUN_20260309_211
```

Interpretation:

1. inspect identifies first failed node and missing artifact,
2. replay confirms whether failure repeats,
3. diff localizes drift scope,
4. remediation targets the smallest confirmed scope.

## Imported runs and replay mismatches

When debugging imported runs:

- verify imported provenance class before selecting baseline,
- treat missing lineage/payload as evidence gaps,
- do not claim equivalence when replay is incomplete.

Replay mismatch on imported evidence may indicate provenance boundary issues, not only graph/runtime defects.

## Next reading

- Baseline selection from history: [Run History](../03-user-guide/04-run-history.md)
- Replay class semantics: [Replay](../03-user-guide/05-replay.md)
- Diff scope interpretation: [Diff](../03-user-guide/06-diff.md)
