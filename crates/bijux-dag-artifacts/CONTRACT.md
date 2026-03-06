# bijux-dag-artifacts contract

## Responsibility
`bijux-dag-artifacts` owns artifact data models and artifact persistence APIs.

## Scope
- Run manifest and node trace model types
- Artifact path/index construction and integrity helpers
- Artifact store and filesystem-backed persistence entrypoints

## Boundary
- Runtime must use artifact persistence through this crate's stable APIs.
- Runtime must not reimplement manifest/index write semantics internally.
- This crate must not depend on app or CLI orchestration layers.
