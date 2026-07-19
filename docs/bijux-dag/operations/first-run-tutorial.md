---
title: First-Run Tutorial
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# First-Run Tutorial

This tutorial produces one retained local run, inspects its evidence, confirms
cache reuse, replays a selected boundary, and verifies the replay. It uses only
checked-in inputs and writes outputs under `artifacts/`.

## Fastest Proof

From repository root:

```bash
make dag-demo
```

The target validates and executes the file-processing graph, performs a warm
cache run, replays the reporting boundary, and strictly verifies the result.
Evidence is retained under `artifacts/dag-demo/`.

Use the manual path below when you need to understand each boundary or replace
the example paths with your own.

## Prepare The Checkout

```bash
make bootstrap

GRAPH_PATH="evidence/dag/authoring/examples/file-processing-report.dag.json"
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"
RUN_ROOT="./artifacts/first-run-tutorial-runs"
CACHE_ROOT="./artifacts/first-run-tutorial-cache"
```

Cargo output remains under `artifacts/rust/target/` through repository
configuration. The tutorial invokes the workspace binary through Cargo so it
does not depend on a separately installed version.

## Confirm The Public Surface

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- version
cargo run -p bijux-dag-cli --bin bijux-dag -- commands
```

`commands` defaults to the stable operator lane.
Maintainer-only probes such as `capabilities` remain outside this
first-run path and require
`BIJUX_DAG_ENABLE_INTERNAL=1`.

The [Release Boundary](../foundation/release-boundary.md) explains stable,
experimental, simulated, internal, and unreleased lanes. The machine-readable
authority is `contracts/foundation/dag_release_truth_table.v1.json`.

## Validate Before Running

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- validate "${GRAPH_PATH}"
```

Validation proves that the graph is accepted under the current schema and
policy. It does not prove successful execution or artifact integrity.

If you need structural inspection before execution, the repository-tested
`show-effective-graph` explicit-path route can expose nodes, edges, roots,
resources, and output contracts. It is not required for the stable first-run
proof.

## Create A Cold Run

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id first-run-tutorial-cold \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=First Run Tutorial Report"
```

The terminal envelope should report success and this retained directory:

```text
artifacts/first-run-tutorial-runs/run-first-run-tutorial-cold/
```

The command result is an orientation aid. The finalized directory is the
evidence boundary.

## Inspect Retained Evidence

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- explain \
  "${RUN_ROOT}/run-first-run-tutorial-cold"

cargo run -p bijux-dag-cli --bin bijux-dag -- artifact registry \
  "${RUN_ROOT}/run-first-run-tutorial-cold" \
  --json

cargo run -p bijux-dag-cli --bin bijux-dag -- artifact-inspect \
  "${RUN_ROOT}/run-first-run-tutorial-cold" \
  render_report:report.md \
  --json
```

Check that:

- the manifest records `source_dir` and `report_title`;
- all four nodes reached their expected terminal states;
- the artifact registry contains the rendered report;
- artifact inspection resolves the declared `render_report:report.md` output;
- retained indexes and content digests agree.

[Run Evidence Layout](../interfaces/run-evidence-layout.md) defines which
files are authoritative and which are summaries.

## Confirm Warm Reuse

Run the same graph with identical inputs and cache root:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id first-run-tutorial-warm \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=First Run Tutorial Report"
```

The upstream processing stages should be reused. `render_report` is
intentionally regenerated. Cache reuse is valid only when the retained
identity and integrity checks admit it; a fast second run alone is not proof.

## Replay And Verify

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

Replay verifies retained inputs crossing into the selected boundary before
rerunning `render_report`. Strict verification then checks that the replayed
run still satisfies its retained contracts. The [Replay
Contract](../../spec/REPLAY_CONTRACT.md) defines the claim boundary.

## Completion Evidence

The tutorial is complete when you can point to:

- accepted graph validation;
- a finalized cold run with recorded effective inputs;
- an inspectable report artifact;
- a warm run with explainable cache decisions;
- a replay carrying source and parent identity;
- a successful strict verification result.

If any item is missing, state that gap rather than describing the demonstration
as end-to-end proof.

## Next Reads

- [File Processing Workflow](file-processing-workflow.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Cache Behavior Workflow](cache-behavior-workflow.md)
- [Execution Security And Isolation](security-isolation-truth.md)
- [Installation And Setup](installation-and-setup.md)
