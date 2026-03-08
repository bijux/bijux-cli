# Architecture documentation

Audience: maintainers and architects.
Owner: architecture group.
Status: stable.

## Architecture entrypoint

Canonical architecture map and boundary reference starts here.

## What this section contains

- Living architecture boundary maps
- Crate and module responsibility boundaries
- Runtime architecture and control-plane model narratives
- Integration boundaries for scheduling, storage, trust, and orchestration
- Design decisions that affect ongoing implementation direction

This section is for structure, boundaries, and operating constraints.
It must not contain historical governance process notes or release planning.

## Current architecture maps

- [Architecture map index](./README.md)
- [Runtime execution flow](./runtime-execution-flow.md)
- [Runtime core architecture](./runtime_core_architecture.md)
- [Crate ownership map](./module_ownership_map.md)
- [Control-plane and backend model](./dev-control-plane.md)
- [Boundary ownership and module contracts](./module_ownership_map.md)
- [Storage ownership boundaries](./storage-layout-ownership.md)
- [Local and remote runtime model](./local_only_vs_remote_coordinated_runtime.md)

## Canonical migration source

Operational semantics and guarantees for contracts live in `docs/spec/`.
Use `docs/spec/` for normative guarantees; use `docs/architecture/` for structure.
