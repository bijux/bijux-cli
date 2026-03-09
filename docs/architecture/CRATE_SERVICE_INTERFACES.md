# Crate Service Interfaces

## Runtime service interfaces
- `RuntimeExecutionService`
- `RuntimeArtifactService`

These formalize runtime orchestration and artifact persistence boundaries without leaking internal modules.

## Artifact service interfaces
- `RunArtifactStore`
- `RunArtifactVerifier`

These formalize run-dir write/verify behaviors for consumers inside runtime/app control flow.
