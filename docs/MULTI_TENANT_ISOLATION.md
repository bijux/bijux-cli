# Multi-tenant isolation contracts

This document defines tenant isolation contracts across registry, scheduler, run state, artifacts, metrics, lineage, policy, and plugin usage.

## Tenant identity and namespacing

- `TenantId` is a first-class typed identifier.
- DAG names are tenant-scoped via `TenantScopedDagName` (`tenant + namespace + logical name`).
- Run lookup and indexing use tenant-aware composition (`tenant::run_id`) and scoped index keys.

## Ownership and lifecycle

Ownership metadata is explicit for all tenant-owned entities.

Tenant lifecycle states:

- active
- suspended
- restricted
- retiring
- deleted

## Tenant overlays, quotas, and budgets

- deterministic tenant config overlays merge with global defaults.
- tenant concurrency quotas cover runs, nodes, and backfills.
- tenant resource budgets cover CPU, memory, storage, artifact volume, and schedule pressure.
- tenant retention policy covers artifacts, logs, and audit records.

## Isolation boundaries

- queue isolation and tenant-aware scheduler admission control
- tenant-level policy bundle references
- tenant-scoped observability views
- tenant-scoped lineage traversal
- tenant registry partitioning and indexing
- tenant secret scopes
- tenant plugin allowlists
- tenant environment overlays for backend/executor/storage classes

## Provisioning and bootstrap

`TenantProvisioningSpec` defines namespace, registry partition, queue isolation baseline, and default policy bundle.

Bootstrap steps are deterministic and auditable.

## Conformance

`validate_tenant_isolation` produces `TenantIsolationConformanceReport` covering:

- API isolation
- scheduler isolation
- artifact isolation
- metrics isolation
- lineage isolation

Fixture example:

- `crates/bijux-dag-runtime/tests/fixtures/tenancy/isolation_matrix.json`
