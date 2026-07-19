---
title: Lifecycle Overview
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Lifecycle Overview

Use this page when you want the shortest honest explanation of what a DAG run
must pass through before it is safe to trust, inspect, replay, or compare.

The point of the lifecycle is simple: DAG work is not finished when nodes run.
It is finished when the run leaves behind enough evidence to explain what
happened and enough structure to compare that result against another run.

## The Lifecycle In Plain Language

| Stage | What happens | Why it matters |
| --- | --- | --- |
| define | a workflow is parsed, validated, canonicalized, and fingerprinted | broken graph truth should fail before execution starts |
| plan | runtime policy and scheduler state decide what is eligible to run | execution intent must be explicit before work begins |
| execute | nodes run, fail, skip, retry, or short-circuit under policy control | the runtime must capture outcomes rather than hide them |
| retain evidence | manifests, traces, artifacts, and lineage records are written | operators need durable proof after the process exits |
| replay and compare | later runs are evaluated against retained baselines | comparison is how the stack distinguishes stable behavior from drift |
| decide | operators use that evidence for release, rollback, investigation, or repair | the product is meant to support decisions, not just execution |

## Lifecycle Stages

1. definition parse, validate, canonicalize, and fingerprint
2. planning and scheduler eligibility computation
3. node execution and outcome capture
4. run and artifact persistence with lineage links
5. replay and diff classification against baselines
6. operator release, rollback, or incident decision

## What This Lifecycle Is Not Saying

- It is not claiming every backend exposes identical behavior.
- It is not saying execution success alone is enough to trust a run.
- It is not replacing the architecture pages when you need scheduler or replay
  internals.

## Code Anchors

- `crates/bijux-dag-core/src/pipeline/`
- `crates/bijux-dag-runtime/src/runtime_core/`
- `crates/bijux-dag-artifacts/src/storage/`
- `crates/bijux-dag-app/src/replay/`
- `crates/bijux-dag-app/src/routes/`

## Continue Reading

- [Execution Model](../architecture/execution-model.md)
- [Common Workflows](../operations/common-workflows.md)
- [Failure Recovery](../operations/failure-recovery.md)
