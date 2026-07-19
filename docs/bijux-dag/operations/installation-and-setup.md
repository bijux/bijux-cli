---
title: Installation And Setup
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Installation And Setup

Choose one execution mode before troubleshooting setup:

- install `bijux-dag-cli` when you need the released `bijux-dag` command
- use `cargo run` from a checkout when developing or verifying repository
  examples against the current source

Do not build from source and then invoke an unrelated `bijux-dag` found on
`PATH`. Executable identity is part of a trustworthy setup result.

## Toolchain Requirements

The workspace pins Rust `1.86.0` with `rustfmt` and `clippy` in
`rust-toolchain.toml`. Cargo output is redirected to
`artifacts/rust/target/` by repository configuration.

The stable local shell lane requires only the host tools used by the graph.
The retained file-processing example invokes `python3`. Container nodes
additionally require a supported container engine. Kubernetes and SLURM
backends require their documented command-line clients and shared-storage
contracts; they are not prerequisites for local shell execution.

## Install The Released Command

```bash
cargo install bijux-dag-cli
command -v bijux-dag
bijux-dag version
bijux-dag commands
```

`bijux-dag-cli` is the package name; `bijux-dag` is the installed binary.
`commands` defaults to the stable operator lane. Confirm that `command -v`
resolves the installation you intended before relying on the result in
automation.

The root `bijux` runtime can report official product ownership:

```bash
bijux apps which dag --format json --no-pretty
bijux apps version dag --format json --no-pretty
```

Those commands describe delegation. The standalone `bijux-dag` binary remains
the public DAG operator surface.

## Run From A Source Checkout

From the repository root:

```bash
make bootstrap
cargo run -p bijux-dag-cli --bin bijux-dag -- version
cargo run -p bijux-dag-cli --bin bijux-dag -- commands
```

`cargo run -p bijux-dag-cli --bin bijux-dag -- ...` binds every example to the
current checkout. A release build alone does not add
`artifacts/rust/target/release/` to `PATH`; invoke that binary by its explicit
path if release-mode behavior is the subject of the check.

## Repository Proof Path

Run:

```bash
make dag-demo
```

This is stronger than `--help` or a package test. It validates a checked-in
graph, inspects the effective graph, creates a cold retained run, verifies its
artifact, performs a warm cache run, replays the reporting boundary, and
finishes with strict verification. Evidence is retained under
`artifacts/dag-demo/`.

The demo proves the current binary against one repository-owned workflow. It
does not prove arbitrary graph correctness, hostile-code isolation, or backend
availability outside the local environment.

## Manual Smoke Run

Use the manual path when the individual proof boundaries matter:

```bash
GRAPH_PATH="evidence/dag/authoring/examples/file-processing-report.dag.json"
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"
RUN_ROOT="./artifacts/setup-runs"

cargo run -p bijux-dag-cli --bin bijux-dag -- validate "${GRAPH_PATH}"

cargo run -p bijux-dag-cli --bin bijux-dag -- run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id setup-file-processing \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=Setup Verification Report"

cargo run -p bijux-dag-cli --bin bijux-dag -- artifact-inspect \
  "${RUN_ROOT}/run-setup-file-processing" \
  render_report:report.md \
  --json

cargo run -p bijux-dag-cli --bin bijux-dag -- verify --json \
  "${RUN_ROOT}/run-setup-file-processing" \
  --strict
```

Validation proves graph acceptance only. The run proves that this environment
executed the graph. Artifact inspection proves that one declared output can be
resolved. Strict verification checks retained evidence consistency. Keep these
claims separate when reporting setup success.

## Failure Classification

| Failure | Check | Correct owner |
| --- | --- | --- |
| `bijux-dag` not found | `command -v bijux-dag` and Cargo install output | installation and `PATH` |
| wrong version | resolved binary path and `bijux-dag version` | duplicate or stale installation |
| source command runs different code | use `cargo run -p bijux-dag-cli --bin bijux-dag --` | checkout selection |
| graph rejected | validation error and `configs/dag/schema/dag.schema.json` | graph contract |
| required input missing | graph `inputs` declaration and supplied `--input` values | invocation |
| node process missing | retained node trace and host `PATH` | graph execution environment |
| container engine unavailable | node failure class and engine discovery | container backend |
| retained output absent or inconsistent | artifact registry and strict verification | artifact production or persistence |

Do not reinstall the command to address a schema or graph-input failure.
Likewise, do not edit a graph to hide a missing host executable. Route the
failure to the layer that owns it.

## Setup Completion

A usable environment has:

1. an identified executable and version
2. a visible stable command inventory
3. a known-good graph that validates
4. a finalized run under `artifacts/`
5. an inspectable declared output
6. successful strict evidence verification

For source development, `make dag-demo` supplies this proof. For an installed
binary, run an equivalent retained workflow owned by your deployment and keep
its graph, inputs, command envelope, run directory, and verification result.

## Related Guides

- [First-Run Tutorial](first-run-tutorial.md)
- [Release Boundary](../foundation/release-boundary.md)
- [Execution Security And Isolation](security-isolation-truth.md)
- [Run Evidence Layout](../interfaces/run-evidence-layout.md)
- [Failure Recovery](failure-recovery.md)
