---
title: Container Packaging Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# Container Packaging Workflow

This guide demonstrates a real container-backed DAG workflow that validates a
source note on the host, passes it into a container node, and emits a bundled
release note plus recorded container identity.

The workflow is backed by
`evidence/dag/authoring/examples/release-note-bundle.dag.json` and the sample
source note under
`evidence/dag/authoring/examples/release-note-source/weekly-update.md`.

## What This Workflow Proves

- a graph input path is validated on the host before container execution
- the container node receives mounted upstream inputs
- the container node writes retained outputs under its run directory
- stdout and stderr from the container step are captured
- the retained trace records container engine, version, and image digest
- a missing container engine fails as a clear infrastructure error

## Prepare The Run

Run these commands from repository root:

```bash
GRAPH_PATH="evidence/dag/authoring/examples/release-note-bundle.dag.json"
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/release-note-source/weekly-update.md"
RUN_ROOT="./artifacts/release-note-bundle-runs"
```

This workflow expects a local Docker-compatible engine on `PATH`. If the
selected image is not already present, the local engine may pull it during the
run.

## Validate The Graph

```bash
bijux-dag validate "${GRAPH_PATH}"
```

## Run The Container Workflow

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id release-note-bundle \
  --input "source_note=${SOURCE_NOTE}" \
  --input "bundle_label=Release Brief"
```

The retained run writes the container-backed outputs under:

```text
artifacts/release-note-bundle-runs/run-release-note-bundle/nodes/package_bundle/outputs/bundle/
```

## Inspect The Retained Outputs

The workflow emits two retained outputs from the container node:

- `release-note.txt`, the bundled note with the supplied label at the top
- `container-summary.json`, the recorded bundle label, source byte count, and
  container workdir

Inspect them directly:

```bash
cat "${RUN_ROOT}/run-release-note-bundle/nodes/package_bundle/outputs/bundle/release-note.txt"
cat "${RUN_ROOT}/run-release-note-bundle/nodes/package_bundle/outputs/bundle/container-summary.json"
```

The retained stdout and stderr are also preserved:

```bash
cat "${RUN_ROOT}/run-release-note-bundle/nodes/package_bundle/stdout.log"
cat "${RUN_ROOT}/run-release-note-bundle/nodes/package_bundle/stderr.log"
```

## Inspect The Recorded Container Identity

Use `explain` for the run-level view and inspect the retained node trace when
you need the exact engine and image identity:

```bash
bijux-dag explain "${RUN_ROOT}/run-release-note-bundle"
cat "${RUN_ROOT}/run-release-note-bundle/nodes/package_bundle/trace.json"
```

The trace records:

- `adapter_id = "container"`
- `container.engine = "docker"`
- `container.engine_version`
- `container.image`
- `container.image_digest`

## Failure Mode When Docker Is Missing

When Docker is not available on `PATH`, the run does not degrade into a shell
fallback. The `package_bundle` node fails as an infrastructure error and the
retained trace records `CONTAINER_ENGINE_UNAVAILABLE`.

Use that failure mode when diagnosing a workstation or CI environment where
the graph is valid but the container runtime is not installed or not
discoverable.

## Reading Rule

Use this guide when the question is whether a container node in `bijux-dag`
actually receives mounted inputs, writes retained outputs, and records engine
identity rather than acting as a decorative graph marker.

## Next Reads

- [Common Workflows](common-workflows.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Deployment Boundaries](deployment-boundaries.md)
- [Execution Security And Isolation](security-isolation-truth.md)
