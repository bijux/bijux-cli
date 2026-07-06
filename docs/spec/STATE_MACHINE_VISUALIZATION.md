---
title: State Machine Visualization
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# State Machine Visualization

This visualization mirrors the lifecycle contract in
`docs/spec/STATE_MACHINE_CONTRACT.md`.

## Node lifecycle

```mermaid
stateDiagram-v2
    [*] --> pending
    pending --> eligible
    pending --> skipped
    pending --> cancelled
    pending --> timed_out
    eligible --> queued
    eligible --> skipped
    eligible --> cached
    eligible --> cancelled
    eligible --> timed_out
    queued --> running
    queued --> skipped
    queued --> cached
    queued --> failed
    queued --> cancelled
    queued --> timed_out
    running --> success
    running --> failed
    running --> cancelled
    running --> timed_out
```

## Run lifecycle

```mermaid
stateDiagram-v2
    [*] --> submitted
    submitted --> planning
    planning --> running
    running --> paused
    paused --> running
    running --> interrupted
    interrupted --> running
    interrupted --> cancelling
    running --> cancelling
    cancelling --> cancelled
    running --> failed
    running --> succeeded
    running --> timed_out
```

## Verification anchors

- `validate_node_transition`
- `validate_run_transition`
- `verify_post_run_state_consistency`
- `terminal_transition_audit_events`
- `run_dag_verify_state`
