# bijux-dag-artifacts contract

Responsibility: Run artifact models, persistence services, integrity proofs, and lifecycle policy helpers.

## Responsibility
`bijux-dag-artifacts` owns artifact data models and artifact persistence APIs.
It is explicitly a `format + IO` crate.

## Scope
- Run manifest, node trace, and outputs index model types (`models`)
- Artifact path construction helpers (`paths`)
- Artifact hashing/index/integrity helpers (`hash`, `index`, `proof`, `schema`)
- Artifact store and filesystem-backed persistence entrypoints (`store`, root write helpers)
- Artifact lifecycle policy models (`retention`, `promotion`, `lineage`)
- `src/lib.rs` is the only root Rust file; module logic must live in bounded domain folders.

## Internal boundaries
- `storage/*`: authoritative artifact models, hardening, and service orchestration.
- `io/*`: filesystem-backed read/write surfaces.
- `integrity/*`: hashing, proof, schema, and index validation surfaces.
- `layout/*`: path and platform layout helpers.
- `lifecycle/*`: retention, promotion, and lineage policy surfaces.

## Boundary
- Runtime must use artifact persistence through this crate's stable APIs.
- Runtime must not reimplement manifest/index write semantics internally.
- This crate must not depend on app or CLI orchestration layers.
