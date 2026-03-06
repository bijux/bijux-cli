# Geo federation disaster recovery playbook

## Regional control-plane outage

1. Detect outage scope and affected ownership leases.
2. Fence stale regional writers.
3. Promote designated failover region ownership.
4. Reconcile scheduler queue ownership and schedule evaluation checkpoints.
5. Validate registry write routing before resuming activation changes.

## Regional artifact-store outage

1. Freeze promotion and replication-dependent release actions.
2. Activate artifact replication fallback reads from healthy region.
3. Rebuild replication backlog index after storage recovery.
4. Run lineage consistency checks for cross-region ancestry.
5. Restore replication policy targets and monitor lag burn-down.

## Split-brain mitigation

- Rotate fencing tokens.
- Freeze secondary writers.
- Reconcile authoritative log.
- Resume writes only after ownership convergence checks pass.
