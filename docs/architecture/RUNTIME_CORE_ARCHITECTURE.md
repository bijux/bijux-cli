# Runtime Core Architecture

## Core execution layers
- `runtime` facade: stable core runtime orchestration entrypoints
- `execution` facade: plan/backend/executor path
- `adapters` facade: adapter boundary and conformance surfaces

## Sacred flow
1. graph parse and validation outside runtime core
2. planner lowers graph into executable plan
3. scheduler determines deterministic readiness
4. engine executes via backend and policy boundaries
5. run/artifact state finalization and invariants

## Ownership boundaries
- core runtime modules own deterministic execution semantics
- support modules own policy and contract support logic
- speculative modules remain internal and must not widen public runtime API

## Freeze rule
New runtime source modules are blocked by `runtime-module-triage` governance unless explicitly added to `configs/policy/runtime_module_freeze.json` with ownership rationale.
