# Deployment backends and capability matrix

## Supported backend contract variants

- local
- subprocess
- container
- kubernetes
- hpc
- external service

Local runtime remains the correctness reference backend.
Only local/subprocess are implemented as runnable backend paths in this repo.
Container, kubernetes, and hpc entries are contract-model surfaces and simulation
targets until dedicated backend implementations land.

## Capability negotiation

Planner-side capability negotiation validates backend support before execution:

- container execution
- network isolation
- environment allowlists
- artifact mounts
- remote logs
- gpu requirements

Backends that do not satisfy declared requirements must be rejected before dispatch.

## Backend-specific execution contracts

- Container contract: image, command/args, mount path, environment allowlist.
- Kubernetes contract: namespace, pod template, image resolution, mount strategy, log collection.
- HPC contract: queue, scheduler submit/poll/cancel commands.

## Secrets and artifacts

- Secret injection contracts are separate from standard environment allowlists.
- Artifact transport contracts support local copy, hardlink, remote upload, and remote download.
- Object storage contracts cover S3/MinIO-style configuration.

## Queue isolation and multi-tenant identity

- Queue partitioning is represented by queue + optional tenant identity + concurrency cap.
- Tenant identity model includes tenant id, namespace, and labels.

## Scaling and availability

- Scheduler scaling plan declares worker counts and sharding keys.
- High-availability scheduler plan declares leader election and durable queue strategy.

## Conformance fixture

Backend conformance fixture:

- `benchmarks/fixtures/infrastructure/backend_conformance_matrix.json`
