# Adapters

Adapters are the meaning boundary between runtime semantics and backend execution systems.

## Adapter contract

An adapter must:

- accept runtime work units without changing graph semantics,
- execute via backend-native mechanisms,
- return canonical outcome envelopes,
- declare capability support and gaps explicitly.

An adapter must not redefine run/artifact identity rules or diff/replay vocabulary.

## Backend-family constraints

### Local process backends

- strong observability,
- lower isolation guarantees,
- environment drift often tied to host/toolchain changes.

### Containerized backends

- stronger isolation,
- additional image/runtime compatibility surface,
- reproducibility depends on image pinning discipline.

### Remote/job backends

- queue/network failure classes,
- asynchronous result reconciliation,
- capability parity must be declared, not assumed.

## Adapter versus runtime responsibilities

| Concern | Adapter owns | Runtime owns |
| --- | --- | --- |
| Backend invocation | translation and backend call mechanics | dispatching ready nodes |
| Result shaping | native -> canonical mapping | state transitions on canonical outcomes |
| Capability declaration | feature support/limits | applying limits to guarantees and classification |
| Scheduling policy | must not define | dependency-correct ordering |
| Identity semantics | must preserve | defines graph/run/artifact contracts |

## Next reading

- Backend support classes: [Backend Support](../07-operations/05-backend-support.md)
- Adapter implementation details: [Adapter Development](../08-development/03-adapter-development.md)
- Artifact/run contracts: [Artifact Model Specification](../06-specification/03-artifact-model.md), [Run Model Specification](../06-specification/02-run-model.md)
