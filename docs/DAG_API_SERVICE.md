# dag-api service boundary plan

This document defines the future service boundary for `dag-api` so the repository control plane can evolve into a networked control plane without semantic drift.

## Service boundaries

- API boundary: typed request/response resources for DAGs, runs, artifacts, schedules, queue, policy, and audit.
- Scheduler boundary: durable schedule evaluation and run submission.
- Registry boundary: DAG publication workflow and compatibility selection.
- Executor boundary: run-control delegation and execution status ingestion.

## Typed resource model

`dag-api` resources:

- DAG
- DAG version
- run
- node attempt
- artifact
- schedule
- queue
- policy
- audit event

## Typed operation model

Registry APIs:

- publish
- validate
- activate
- deprecate
- retire
- inspect

Run-control APIs:

- submit
- cancel
- pause
- resume
- retry
- replay
- verify

Artifact APIs:

- inspect
- export
- verify
- lineage
- retention action

Schedule APIs:

- create
- update
- suspend
- preview
- audit

## Pagination, filtering, and optimistic concurrency

- list endpoints use `Pagination { limit, cursor }`.
- list endpoints support typed `ListFilter { field, value }`.
- mutable resources include `VersionedResource { resource_version, etag }`.

## Authentication and authorization

Authentication principals:

- CLI user
- service account
- worker identity

Authorization uses action + scoped rule matching against typed resource prefixes.

## Configuration and subscriptions

- environment-scoped configuration supports base values and overlays.
- event subscriptions are first-class typed resources for webhook/stream evolution.

## API compatibility rules

- major versions are explicit and bounded by compatibility policy.
- minor versions allow additive fields only.
- compatibility checks are deterministic (`check_api_compatibility`).

## CLI as a future SDK consumer

`bijux-dev-dag` is expected to become a thin client over `dag-api` with typed model parity.

## Minimal MVP definition

Control-plane MVP includes:

- registry publication and inspection
- run-control operations
- schedule management and preview
- audit persistence

Control-plane MVP excludes:

- distributed worker orchestration
- advanced HA/sharding behavior
