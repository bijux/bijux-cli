# API resource contract draft

This document defines the minimal service control-plane resource model for the future `dag-api`.

## Resources

- DAGs
- DAG versions
- runs
- node attempts
- artifacts
- schedules
- queues
- policies
- audit events

## Control-plane operations

- registry: publish, validate, activate, deprecate, retire, inspect
- run-control: submit, cancel, pause, resume, retry, replay, verify
- artifact: inspect, export, verify, lineage, retention action
- schedule: create, update, suspend, preview, audit

## Request/response and list contracts

- typed request envelope: `TypedApiRequest`
- typed response envelope: `TypedApiResponse`
- pagination: `Pagination { limit, cursor }`
- filtering: `ListFilter { field, value }`
- mutable versioning: `VersionedResource { resource_version, etag }`

## Authentication and authorization boundaries

- principals: CLI user, service account, worker identity
- authorization: scoped action evaluation via typed rules

## Environment and subscription contracts

- environment-scoped configuration uses typed values + overlays
- event subscription model is typed for future webhooks/streaming

## Storage and reproducibility

- DAG registry storage is abstracted for filesystem and database implementations.
- Policy bundles are versioned to make decisions reproducible.
- Schedule definitions are separated from execution submissions.

## API compatibility and evolution

- API versions are explicit (`major`, `minor`)
- major versions must satisfy compatibility bounds
- minor versions are additive-only

## CLI compatibility mapping

Current `bijux-dev-dag` commands map to future service operations as follows:

- `checks run` -> repository validation endpoint
- `contracts run` -> contract execution endpoint
- `schedule validate` -> schedule compile endpoint
- `schedule preview` -> schedule simulation endpoint
- `observability-report` -> run observability report endpoint

The mapping keeps command semantics stable when CLI becomes a thin service client.
