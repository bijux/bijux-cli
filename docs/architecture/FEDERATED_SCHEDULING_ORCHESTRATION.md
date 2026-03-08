# Federated scheduling and cross-cluster orchestration

## Federation model

Scheduler domains coordinate through explicit delegation contracts and capability exchange without sharing full internal scheduler state.

## Identity and trust

Each scheduler domain has a stable identity and trust profile. Delegation decisions require trust-compatible domain pairing.

## Delegation and routing

- Parent-to-child run handoff is represented by deterministic delegation records.
- Cross-cluster routing can consider region, tenant, backend class, and data locality.
- Domain routing explanations preserve evidence for why a domain was selected.

## Peering and overflow

Peering rules define controlled overflow and burst sharing between domains. Inter-domain flow control prevents delegation storms.

## Federated backfills and suppression

Backfill workloads can be partitioned across domains with deterministic partitioning guarantees.

Schedule suppression can be coordinated across domains for maintenance and incident response.

## Replay and failure semantics

Cross-domain replay requires artifact, policy, and backend compatibility checks.

Delegation failures must resolve via explicit policy:
- retry in same domain
- reroute to another domain
- quarantine

## Observability and audit exchange

Peering contracts define metrics and audit event exchange with redaction boundaries.

## Concurrency and trust routing

Federation supports concurrent global and local quotas.

Sensitive workloads are restricted by trust-tier routing rules.

## Maturity and conformance

Federation maturity progression:
- single domain
- active/passive
- overflow peering
- full multi-domain orchestration

Conformance gate requires deterministic routing, delegated-run lineage auditability, and complete audit events.
