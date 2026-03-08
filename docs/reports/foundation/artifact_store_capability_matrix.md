# Artifact Store Capability Matrix

Generated from `crates/bijux-dag-artifacts/src/io/store.rs` backend capability declarations.

| capability | filesystem store | object store model |
|---|---|---|
| write artifact payload | implemented | modeled |
| read artifact payload | implemented | modeled |
| runtime-backed execution | implemented | modeled |

Notes:
- Runtime source-of-truth currently implements filesystem storage semantics.
- Object-store surface remains declared capability only and must not be presented as implemented runtime behavior.
