# Running A Pipeline

This is the first-run operational path from execution to evidence-based validation.

## First-run walkthrough

Start with a validated DAG:

```bash
bijux-dag run --dag ./examples/first-orders.dag.json
```

Capture the returned run ID (example: `RUN_20260309_301`). Then follow this exact sequence:

```bash
bijux-dag inspect run --run-id RUN_20260309_301
bijux-dag inspect artifact --run-id RUN_20260309_301
bijux-dag replay --run-id RUN_20260309_301
bijux-dag diff run --left RUN_20260309_301 --right RUN_20260309_302
```

## What evidence is created during a run

A run materializes evidence that later commands consume:

- run metadata (identity, terminal state, timing envelope),
- node outcomes (success/failure classifications),
- artifact references and lineage links,
- diagnostics needed for inspect and replay analysis.

Conceptual run-directory shape:

```text
runs/RUN_20260309_301/
  run-metadata.json
  node-outcomes/
  artifacts-index.json
  diagnostics/
```

## How to read success

A run is operationally successful when:

- terminal status is succeeded,
- expected nodes have recorded outcomes,
- expected artifacts exist with lineage,
- replay/diff do not report unexpected drift for equivalent context.

## Failure case: immediate debug path

Example: run fails at `summarize_orders` because input file is missing.

Debug sequence:

1. `inspect run` to identify first failed node and reason class.
2. `inspect artifact` to confirm missing upstream artifact.
3. replay baseline (if needed) to separate deterministic defect from transient issue.
4. diff failing run against known-good run to classify change scope.

Do not skip directly to retry loops without inspect evidence.

## Next reading

- Run object semantics and ancestry: [Understanding Runs](../02-getting-started/04-understanding-runs.md)
- Failure-class specific guidance: [Basic Troubleshooting](../02-getting-started/05-basic-troubleshooting.md)
