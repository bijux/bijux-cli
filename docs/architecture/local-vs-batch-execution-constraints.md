# Local vs Batch Execution Constraints

## Local execution
- low-latency lifecycle: prepare -> launch -> observe -> finalize
- direct filesystem artifact access
- immediate status observation
- strong interactive feedback

## Batch/HPC execution
- scheduler-mediated lifecycle with delayed transitions
- explicit submit/poll/cancel interaction model
- remote artifact handoff and delayed availability
- duplicate/stale status delivery must be tolerated

## Shared invariants
- node attempt identity and retry lineage
- deterministic scheduler accounting semantics
- explicit failure classification and observability persistence

## Current boundary
This repository implements local/subprocess execution and batch/HPC simulation
surfaces only. Production batch orchestration remains outside implemented scope.
