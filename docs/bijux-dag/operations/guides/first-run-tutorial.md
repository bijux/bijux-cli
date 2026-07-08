---
title: First-Run Tutorial
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# First-Run Tutorial

This tutorial is the shortest honest path from a fresh checkout to a retained
`bijux-dag` run that you can inspect, rerun from cache, replay, and verify.

It is also the shortest proof path for the `bijux-dag` product promise:
explicit graph contracts at validation time, deterministic execution records in
the retained run, verified artifacts after execution, cache explanation on the
warm rerun, and replayable run bundles through the retained source run.

Use it when the question is not "what can the product eventually do," but
"can I run one real workflow and understand exactly what happened?"

The tutorial uses the repository file-processing example because it stays on
the stable local operator surface while proving:

- runtime graph inputs
- graph inspection before execution
- retained run and artifact evidence
- warm cache reuse on a second run
- focused replay from a selected node
- strict verification of retained outputs

## One-Command Proof

If the immediate question is "does this repository already prove one real DAG
workflow end to end?", run:

```bash
make dag-demo
```

That command writes retained evidence under `artifacts/dag-demo/` and executes
the same workflow this tutorial explains step by step:

- graph inspection before execution
- cold retained run creation
- retained artifact registry and report inspection
- warm cache reuse on the second run
- focused replay of the final reporting boundary
- strict verification of the replayed run

Use the remaining sections of this tutorial when you want to see each command,
understand the run directory layout, or substitute your own run and cache
roots.

## What You Need

You can either install the CLI or run it from the repository checkout.

Install path:

```bash
cargo install bijux-dag-cli
```

Repository path:

```bash
cargo build -p bijux-dag-cli --release
```

The commands below use the repository path so they work on a clean checkout
without relying on an already-installed binary.

## Prepare Variables

Run these commands from repository root:

```bash
GRAPH_PATH="evidence/dag/authoring/examples/file-processing-report.dag.json"
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"
RUN_ROOT="./artifacts/first-run-tutorial-runs"
CACHE_ROOT="./artifacts/first-run-tutorial-cache"
```

## Check The Command Surface

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- version
cargo run -p bijux-dag-cli --bin bijux-dag -- commands
```

You should see the current build identity plus the stable root operator
inventory.

## Inspect The Graph Before Execution

Validate the graph and inspect its structure before any run artifacts are
written:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- validate "${GRAPH_PATH}"

cargo run -p bijux-dag-cli --bin bijux-dag -- show-effective-graph --json \
  "${GRAPH_PATH}"
```

That `show-effective-graph` response is the graph-inspection step. It is a
repository-tested explicit-path inspection route rather than part of the
default stable `--help` surface, and it shows the nodes, edges, roots, leaves,
resources, and output contracts before execution starts. The runtime input
bindings are then proven on the real `run` command below.

## Run The Workflow

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id first-run-tutorial-cold \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --progress compact \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=First Run Tutorial Report"
```

The run writes a retained run directory under:

```text
artifacts/first-run-tutorial-runs/run-first-run-tutorial-cold
```

`--progress compact` is the readable long-run lane. In human mode it keeps a
single live status line on stderr. With `--json`, it also streams
`dag.run.progress` events before the final `dag.run` envelope so automation can
watch elapsed time, active nodes, cache hits, and the latest failure without
waiting for completion.

## Inspect The Run

Inspect the retained run summary:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- explain \
  "${RUN_ROOT}/run-first-run-tutorial-cold"
```

This is the run-inspection step. It should show a successful four-node
workflow with the declared graph inputs recorded in the manifest.

## Inspect The Artifacts

List retained artifacts and inspect the final report:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- artifact registry \
  "${RUN_ROOT}/run-first-run-tutorial-cold" \
  --json

cargo run -p bijux-dag-cli --bin bijux-dag -- artifact-inspect \
  "${RUN_ROOT}/run-first-run-tutorial-cold" \
  render_report:report.md \
  --json
```

The final report payload lives at:

```text
artifacts/first-run-tutorial-runs/run-first-run-tutorial-cold/nodes/render_report/outputs/report/report.md
```

## Rerun And Check Warm Cache Reuse

Run the same workflow again with the same cache directory:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id first-run-tutorial-warm \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --progress compact \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=First Run Tutorial Report"
```

On the warm run, `validate_files`, `transform_files`, and
`aggregate_metrics` should be reused from cache. `render_report` is
intentionally regenerated so the final report stays tied to the retained
summary input rather than a stale cached publication.

## Replay And Verify The Result

Replay only the final reporting boundary and then verify the replayed run:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- replay --json \
  --source-run-id first-run-tutorial-cold \
  --source-run-root "${RUN_ROOT}" \
  --out "${RUN_ROOT}" \
  --run-id first-run-tutorial-replay \
  --from-node render_report

cargo run -p bijux-dag-cli --bin bijux-dag -- verify --json \
  "${RUN_ROOT}/run-first-run-tutorial-replay" \
  --strict
```

This proves the final boundary can be rerun from retained upstream evidence and
that the resulting run still satisfies strict verification.

## You Are Done When

You have completed the first-run tutorial when all of these are true:

- `validate` accepts the file-processing graph
- `show-effective-graph` shows the graph structure before execution
- the cold run creates a retained run directory under `artifacts/`
- `explain` can read that retained run successfully
- `artifact registry` and `artifact-inspect` show the final report artifact
- the warm run reuses upstream stages from cache
- `replay` reruns the final reporting boundary
- `verify --strict` succeeds on the replayed run

## Next Reads

- [File Processing Workflow](file-processing-workflow.md)
- [Evidence-Backed Bulletin Workflow](evidence-backed-bulletin-workflow.md)
- [Operator Workflows](../../interfaces/operator-workflows.md)
- [Installation And Setup](../installation-and-setup.md)
