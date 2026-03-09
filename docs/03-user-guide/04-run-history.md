# Run History

Run history is where you decide whether a failure is new, recurring, or environmental.

## How to read history, not just list it

When scanning history, look for patterns, not single rows:

- first run where behavior changed,
- whether the same node or artifact path keeps drifting,
- whether failures cluster around environment/toolchain changes.

This turns run history into a diagnostic tool, not a timeline dump.

## Regression investigation example

Workflow:

1. list recent runs and identify latest failing run,
2. select the last known-good baseline,
3. inspect both runs,
4. replay failing context if needed,
5. diff baseline vs failing run.

```bash
bijux-dag run history --limit 30
bijux-dag inspect run --run-id RUN_20260309_211
bijux-dag inspect run --run-id RUN_20260309_204
bijux-dag replay --run-id RUN_20260309_211
bijux-dag diff run --left RUN_20260309_204 --right RUN_20260309_211
```

Interpretation:

- recurring drift at same node suggests deterministic defect,
- intermittent drift correlated with environment changes suggests bounded non-equivalence.

## Ancestry context for practical use

Treat original, replayed, and imported runs as different evidence classes when selecting baselines. Mixing them without provenance awareness weakens conclusions.

## Next reading

- Replay classification meanings: [Replay](../03-user-guide/05-replay.md)
- Diff interpretation strategy: [Diff](../03-user-guide/06-diff.md)
- Formal run semantics: [Run Model Specification](../06-specification/02-run-model.md)
