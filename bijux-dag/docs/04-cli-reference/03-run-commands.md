# Run Commands

Use `run` commands to create execution evidence and navigate run history.

## What this command family is for

`run` commands answer:

- create run: what happened in this execution instance?
- list/history: where is my baseline and where did behavior change?
- inspect handoff: which run ID should replay/diff inspect next?

## Core invocation patterns

```bash
bijux-dag run --help
bijux-dag run --dag ./pipelines/main.dag.json
bijux-dag run history --limit 20 --output json
```

Some builds may expose additional run surfaces (for example summary/timeline/show). Use `bijux-dag run --help` to discover those exact commands before scripting.

## Run ID and imported-run references

Run ID is the primary evidence selector across inspect/replay/diff.

Imported runs should be treated as separate provenance class in history analysis. Do not substitute imported run IDs for native baselines without explicit trust-boundary checks.

## Example operator flow

```bash
bijux-dag run --dag ./pipelines/main.dag.json
bijux-dag run history --limit 10
bijux-dag inspect run --run-id RUN_20260309_220
bijux-dag replay --run-id RUN_20260309_220
```

Expected behavior:

- execution command emits run ID,
- history exposes ordering for baseline selection,
- inspect/replay consume the same run ID as identity anchor.

## Failure modes

- invalid DAG reference,
- run creation failure due to node/runtime errors,
- unknown run ID in history-followup flows.

## Next reading

- Artifact evidence navigation: [Artifact Commands](../04-cli-reference/04-artifact-commands.md)
- Run semantics contract: [Run Model Specification](../06-specification/02-run-model.md)
