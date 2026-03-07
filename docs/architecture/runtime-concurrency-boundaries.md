# Runtime Concurrency Boundaries

```mermaid
flowchart LR
  Plan[ExecutionPlan] --> Scheduler[SchedulerState]
  Scheduler --> Ready[ReadyQueue]
  Scheduler --> Retry[RetryQueue]
  Scheduler --> SEvent[SchedulerEventLog]

  Scheduler --> Engine[Engine Loop]
  Engine --> Coord[RuntimeCoordinationState]
  Coord --> Summary[RunSummaryCounters]
  Coord --> Trace[TraceWriteLedger]
  Coord --> Cache[CacheClaimMap]
  Coord --> Latest[LatestLinkGuard]

  Engine --> Store[RunDir + Cache Store]
  Store --> Manifest[manifest.json]
  Store --> Timeline[observability.timeline.json]
```

## Boundaries

- `SchedulerState` owns readiness transitions and scheduling event ordering.
- `RuntimeCoordinationState` owns concurrent mutation guards shared by worker
  paths.
- Storage writers are invoked after coordination checks and remain outside
  scheduler internals.

## Non-goals

- No lock-free shared mutable state.
- No implicit cross-module mutation without ownership API.
