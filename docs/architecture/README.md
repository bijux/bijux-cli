# Architecture documentation

Audience: maintainers and architects.
Owner: architecture group.
Status: stable.

## Directory role

- Keep only living boundary maps and system structure narratives.
- Move immutable decisions and historical migration notes to `docs/adr/`.
- Keep normative guarantees in `docs/spec/`, not in architecture pages.

## Reading order

1. `RUNTIME_CORE_ARCHITECTURE.md`
2. `RUNTIME-EXECUTION-FLOW.md`
3. `RUNTIME-CONCURRENCY-BOUNDARIES.md`
4. `ENGINE-BACKEND-RESPONSIBILITIES.md`
5. `CONTROLLER_BACKEND_ARTIFACT_BOUNDARY.md`
6. `EXECUTION-MODE-RESPONSIBILITIES.md`
7. `LOCAL_ONLY_VS_REMOTE_COORDINATED_RUNTIME.md`
8. `LOCAL-VS-BATCH-EXECUTION-CONSTRAINTS.md`
9. `DEV-CONTROL-PLANE.md`
10. `CONTROL_PLANE.md`
11. `MODULE_OWNERSHIP_MAP.md`
12. `CRATE-GRAPH.md`
13. `CRATE_SERVICE_INTERFACES.md`
14. `STORAGE-LAYOUT-OWNERSHIP.md`

## Boundaries with other sections

- `docs/spec/`: guarantees and contract semantics.
- `docs/reference/`: operator-facing reference tables and indexes.
- `docs/adr/`: durable decision records and archived intermediate choices.
