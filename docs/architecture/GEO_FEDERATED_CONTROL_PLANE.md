# Geo-distributed control plane and regional federation

## Regional identity and resource semantics

`RegionId` is a first-class identifier applied consistently across scheduler, artifact stores, workers, tenants, and policy overlays.

DAG versions are globally publishable while activation remains region-selective.

## Scheduling and queueing

Schedule evaluation is region-aware with explicit timezone handling, UTC anchoring, and deterministic failover region behavior.

Queue partitions are region-scoped and may be intentionally shared for overflow patterns.

## Affinity, routing, and replication

Region affinity applies to DAGs, runs, artifacts, and tenants.

Write routing separates global visibility from regional write ownership.

Replication policy is explicit per class for artifacts, run metadata, and audit logs.

## Consistency boundaries

Each resource must be classified as:
- strongly consistent
- regionally consistent
- eventually replicated

These boundaries are part of product contract documentation.

## Observability and lineage

Observability is partitioned by region and aggregated globally with attribution preserved.

Lineage is region-aware so cross-region ancestry remains queryable.

## Reliability and DR

The model includes:
- regional control-plane replicas with durable ownership
- cross-region failover rules
- split-brain detection and fencing-first mitigation
- disaster recovery playbooks for control-plane and artifact-store regional outages

## Migration and readiness

Region migration workflows cover tenants, DAGs, and schedules.

Geo-ready gates require registry, scheduler, lineage, and observability readiness.
