---
title: Trust Boundaries
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Trust Boundaries

`bijux-dag` is a local DAG runtime with explicit evidence, cache, replay, and
artifact trust boundaries.

## Trust what is implemented

- local run execution and artifact production
- manifest, trace, and outputs index verification
- cache proof validation and corruption refusal
- replay and diff classification for locally governed run directories

## Do not assume what is not promised

- Kubernetes orchestration ownership
- HPC scheduler integration ownership
- promoted remote coordination
- distributed consensus for run-state authority

## Operator rule

If a surface is simulated, experimental, or future-facing, treat it as a
modeled contract boundary unless the release boundary explicitly promotes it.
