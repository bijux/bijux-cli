---
title: Operator Command Index
audience: operators
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Operator Command Index

Use these commands when the run already exists and the job is to inspect,
verify, compare, or diagnose it.

## Run inspection commands

- `dag runs list`: enumerate available runs under an explicit root
- `dag runs show`: show compact status and timing for one run
- `dag runs inspect`: derive the structured inspection summary for one run
- `dag runs tree`: render node structure from run evidence
- `dag runs timeline`: render ordered execution events from node traces
- `dag runs diff`: compare two run directories
- `dag runs verify`: verify run integrity and compatibility
- `dag runs doctor`: diagnose corrupt or incomplete run evidence
- `dag runs explain-failure`: explain the root failure boundary
