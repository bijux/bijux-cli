# Understanding Runs

## Purpose
Explain what a run represents and how to reason about run data during normal operation.

## Context
This document provides the mental model needed to interpret run outputs and troubleshooting signals.

## Explanation
A run is a single execution instance of a DAG under concrete inputs and environment conditions.

Run model basics:
- each run has a unique run identifier
- run state transitions reflect execution progress
- run directory persists operational context and evidence

Practical run interpretation:
- successful run: all required nodes completed
- failed run: one or more nodes failed with diagnostic signals
- repeated run IDs should never be reused for distinct executions

Run directory layout (conceptual):
- run metadata
- node execution outputs
- diagnostics and status records
- artifact linkage information

This layout supports inspect, replay, and diff workflows without external reconstruction.

## Examples
```bash
# View run summary
bijux-dag inspect run --run-id RUN_20260309_001

# View run history index (if available in your surface)
bijux-dag run history --limit 10
```

```text
Key fields to watch first:
- run_id
- status
- started_at / finished_at
- failed_node_count
```

## Guarantees
- Run semantics in this guide align with getting-started operational flows.
- Run identity and state concepts are defined once and reused consistently.

## Limitations
- This document does not define formal run schema contracts.
- Backend-specific run storage differences are not covered here.

## Related
- `docs/02-getting-started/03-running-a-pipeline.md`
- `docs/03-user-guide/04-run-history.md`
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/05-run-identity.md`
