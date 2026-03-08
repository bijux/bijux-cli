# Artifact Identity Contract

This contract defines the authoritative artifact identity model for `bijux-dag`.

## Identity type

- `ArtifactId` is a first-class type in `crates/bijux-dag-artifacts/src/integrity/index.rs`.
- Canonical string form: `<node_id>:<file_name>`.
- Identity explanation output is emitted by `dag artifact-inspect` via
  `crates/bijux-dag-app/src/lib.rs::inspect_artifact`.

## Fingerprint composition

Artifact fingerprint is composed from:

- content digest (`sha256`)
- producing `run_id`
- producing `node_id`
- producing `node_fingerprint`
- logical artifact path within the run directory

Implementation anchors:
- `crates/bijux-dag-app/src/lib.rs::inspect_artifact`
- `crates/bijux-dag-artifacts/src/integrity/hash.rs`
- `crates/bijux-dag-artifacts/src/storage/models.rs` (`RunOutputFile`)

## Provenance links

Artifact inspection surfaces must include provenance links to:

- graph fingerprint
- run id
- node id and node fingerprint
- execution attempt number

## Logs policy

- `stdout.log` and `stderr.log` are retained as run evidence and diagnostics artifacts.
- Logs are never treated as canonical semantic outputs for replay equivalence.

## Store capabilities

Current capability status:

- filesystem store: implemented
- object store: modeled-only (not implemented in runtime)
