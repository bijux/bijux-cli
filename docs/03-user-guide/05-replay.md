# Replay

Replay is a validation workflow: it creates candidate evidence and tells you whether behavior stayed equivalent, drifted, or became incomplete.

## Replay as workflow, not command trivia

Use replay when a decision depends on reproducibility:

1. choose trusted baseline run,
2. run replay in target context,
3. classify result,
4. investigate with diff when classification is not equivalent.

## Outcome classes in operator language

- exact equivalence: required outcome surfaces match.
- bounded equivalence: outcomes match within declared capability limits.
- drift: contract-relevant differences detected.
- incomplete: replay could not verify required surfaces.

Treat bounded equivalence as conditional acceptance, not unconditional success.

## Investigation example

```bash
bijux-dag replay --run-id RUN_20260309_204
bijux-dag diff run --left RUN_20260309_204 --right RUN_20260309_221
```

Example interpretation:

- replay status: drift,
- diff scope: artifact hash mismatch in downstream report,
- likely next step: inspect producer node and upstream input changes.

## What replay success does not prove

Replay equivalence does not prove:

- identical wall-clock performance,
- universal cross-backend portability,
- equivalence of external side effects not captured as artifacts.

## Next reading

- How to choose baselines from history: [Run History](../03-user-guide/04-run-history.md)
- How to classify replay differences: [Diff](../03-user-guide/06-diff.md)
- Formal replay contract: [Replay Semantics Specification](../06-specification/07-replay-semantics.md)
