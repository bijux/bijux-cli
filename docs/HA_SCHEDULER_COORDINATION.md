# HA scheduler durable coordination contracts

This document defines high-availability scheduler contracts for leader election, durable queueing, failover ordering, fencing, and deduplication.

## Durable scheduler state

- `DurableSchedulerStateStore` is separate from transient in-memory scheduler state.
- durable records include leader state, queue entries, and tick history.
- in-flight dispatch records persist node dispatch ownership across failover.

## Leader election, epoch, and fencing

- `LeaderElectionState` tracks active leader lease and epoch.
- `SchedulerEpoch` increments on leadership transition.
- `SchedulerFenceToken` prevents stale leaders from mutating durable state.

## Queue durability, sharding, and ownership transfer

- durable queue entries are tenant-aware and timestamped.
- shard lease model supports queue shard ownership.
- queue ownership transfers are explicit, timestamped audit facts.

## Idempotency and deduplication

- run creation is idempotent via deterministic dedup keys.
- cross-replica dedup checks prevent duplicate scheduled runs.
- run submission ordering under failover is deterministic by timestamp and stable tiebreaks.

## Clock assumptions and recovery objectives

- distributed clock assumptions define max skew and tick grace.
- recovery objectives define cold-restart and failover RTO bounds.

## Simulation and conformance

- HA simulation scenarios model replicas, shards, and trigger storms.
- conformance report verifies:
  - no duplicate runs
  - stale leader fencing
  - submission sequencing preservation

Fixtures:

- `benchmarks/fixtures/scheduling/ha/split_brain_failover.json`
- `benchmarks/fixtures/scheduling/ha/trigger_storm_rebalance.json`
- `benchmarks/fixtures/scheduling/ha/cold_restart_objective.json`

## Minimal HA milestone

The minimal HA milestone requires durable queueing, epoch+fencing enforcement, and duplicate-run prevention. It explicitly excludes multi-zone and geo-distributed scheduling.
