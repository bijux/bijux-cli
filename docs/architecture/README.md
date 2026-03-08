# Architecture documentation

Audience: maintainers and architects.
Owner: architecture group.
Status: stable.

## Directory role

- Keep only living boundary maps and system structure narratives.
- Move immutable decisions and historical migration notes to `docs/adr/`.
- Keep normative guarantees in `docs/spec/`, not in architecture pages.

## Reading order

1. `runtime_core_architecture.md`
2. `runtime-execution-flow.md`
3. `runtime-concurrency-boundaries.md`
4. `engine-backend-responsibilities.md`
5. `controller_backend_artifact_boundary.md`
6. `execution-mode-responsibilities.md`
7. `local_only_vs_remote_coordinated_runtime.md`
8. `local-vs-batch-execution-constraints.md`
9. `dev-control-plane.md`
10. `CONTROL_PLANE.md`
11. `module_ownership_map.md`
12. `crate-graph.md`
13. `crate_service_interfaces.md`
14. `storage-layout-ownership.md`

## Boundaries with other sections

- `docs/spec/`: guarantees and contract semantics.
- `docs/reference/`: operator-facing reference tables and indexes.
- `docs/adr/`: durable decision records and archived intermediate choices.
