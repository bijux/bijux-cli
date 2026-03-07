# bijux-dag-artifacts contract

## Responsibility
`bijux-dag-artifacts` owns artifact data models and artifact persistence APIs.
It is explicitly a `format + IO` crate.

## Scope
- Run manifest, node trace, and outputs index model types (`models`)
- Artifact path construction helpers (`paths`)
- Artifact hashing/index/integrity helpers (`hash`, `index`, `proof`, `schema`)
- Artifact store and filesystem-backed persistence entrypoints (`store`, root write helpers)
- Artifact lifecycle policy models (`retention`, `promotion`, `lineage`)

## Boundary
- Runtime must use artifact persistence through this crate's stable APIs.
- Runtime must not reimplement manifest/index write semantics internally.
- This crate must not depend on app or CLI orchestration layers.
