# Graph Identity Contract

## Graph identity

- `graph_id` is the canonical graph fingerprint.
- `graph_id` is derived as SHA256 over canonical graph JSON bytes.
- Canonicalization normalizes ordering and relative path separators.

## Implementation linkage

- `GraphId` type: `crates/bijux-dag-core/src/lib.rs`.
- Canonicalization entrypoints: `crates/bijux-dag-core/src/graph/canonical.rs`.
- Fingerprint entrypoints: `crates/bijux-dag-core/src/analysis/fingerprint.rs`.
- Topology ordering semantics: `crates/bijux-dag-core/src/graph/topology.rs`.

## Identity-affecting fields

- `spec`
- `meta.*` fields
- `inputs`
- `nondeterminism_allowed`
- all node semantics (kind, params, resources, env, outputs, effects, retry, timeout)
- edges

## Identity-non-affecting fields

- object key order in input JSON
- text formatting and line endings in source file
- node `group` (explicitly excluded from node fingerprint)

## Explain output

- `dag fingerprint --explain`
- `dag hash graph --explain`
- `dag canonical-diff` (machine-readable raw vs canonical diff)

Schema: `configs/schema/graph_fingerprint_explain.schema.json`
Schema: `configs/schema/graph_canonical_diff.schema.json`
