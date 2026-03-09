# Adapters

Define the adapter model and its role in separating runtime semantics from execution backends.

Adapters are the architecture boundary between core runtime intent and environment-specific execution mechanisms.

## Explanation
Adapter responsibilities:
- translate runtime work units to backend/execution substrate actions
- return normalized execution outcomes to runtime
- expose capability and limitation boundaries

Adapter architecture decisions:
- keep adapter interface narrow and normalization-focused.
- isolate backend-specific behavior behind capability descriptors.
- require explicit mapping from backend-native outcomes to canonical runtime states.
- refuse silent capability emulation that would hide non-equivalence.

Adapter model constraints:
- adapters must preserve core runtime semantics where supported
- adapter-specific behavior differences must be explicit
- portability claims are bounded by adapter support contracts

Operational implications:
- same DAG can be evaluated across supported adapters
- equivalence checks require replay/diff validation

## Examples
```text
Runtime intent -> Adapter translation -> Backend execution -> Normalized outcome -> Runtime state update
```

```mermaid
graph LR
  A[Runtime Work Unit] --> B[Adapter Translation]
  B --> C[Backend API or Shell]
  C --> D[Backend Result]
  D --> E[Canonical Outcome Mapping]
  E --> F[Runtime State Update]
```

## Guarantees
- Adapter boundary is explicit and architecture-visible.
- Support and limitation framing is aligned with portability docs.

## Limitations
- Full backend compatibility matrix is in operations docs.
- Adapter interface details are in development/spec docs.

## Related
- `docs/05-system-architecture/03-execution-engine.md`
- `docs/07-operations/05-backend-support.md`
- `docs/08-development/03-adapter-development.md`
- `docs/06-specification/03-artifact-model.md`

## Adapter contract and non-equivalence boundaries

Adapter contract obligations:

- accept runtime work units without changing graph semantics,
- map backend-native outcomes into canonical runtime outcome classes,
- declare capability gaps explicitly so replay/diff can classify bounded equivalence.

Non-equivalence risk appears when backend capabilities differ (timeouts, filesystem model, process isolation, or artifact IO guarantees).

## Backend family constraints

- local-process adapters: strong observability, weaker isolation guarantees.
- containerized adapters: stronger isolation, added image/runtime drift surface.
- remote-job adapters: queuing and network boundaries introduce additional failure classes.

Each family may preserve core semantics while differing in fidelity bounds.

## Adapter responsibilities versus runtime responsibilities

| Concern | Adapter | Runtime |
| --- | --- | --- |
| Backend invocation | Owns translation and invocation | Consumes abstract execution interface |
| Outcome shape | Maps native result to canonical envelope | Uses canonical envelope for state transitions |
| Capability declaration | Publishes supported/unsupported features | Uses declarations for bounded guarantees |
| Scheduling policy | Must not define | Owns dependency-based ordering |
| Artifact semantics | Must not redefine | Owns identity and lineage contracts |
