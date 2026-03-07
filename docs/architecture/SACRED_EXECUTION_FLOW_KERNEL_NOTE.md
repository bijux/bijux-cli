# Sacred Execution Flow In Kernel Terms

## Definition

Sacred execution flow is the non-negotiable deterministic path from canonical graph to immutable run record with verifiable artifacts.

## Kernel stages

1. Canonical parse and validation.
2. Deterministic planning.
3. Deterministic readiness and ordering.
4. Node execution with policy enforcement.
5. Artifact and manifest commit.
6. Run finalization and replay/diff truth surfaces.

## Non-goals

- Product/platform narratives do not redefine these stages.
- Backend wrappers do not alter canonical meaning.

## Related contracts

- `docs/spec/SACRED_EXECUTION_FLOW.md`
- `docs/spec/KERNEL_BOUNDARY_CONTRACT.md`
