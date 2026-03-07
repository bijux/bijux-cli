# Artifact system contracts

## Module boundaries

`bijux-dag-artifacts` is organized into explicit modules:

- `store`: filesystem/object storage backend contracts
- `models`: run artifact schemas and wire models
- `paths`: canonical artifact-relative path helpers
- `index`: artifact ids, aliases, output classes, pack manifest, dedup metrics
- `hash`: content hashing helpers
- `schema`: output schema descriptor and validation hook
- `lineage`: lineage edge model and snapshot writer
- `retention`: retention policy model
- `promotion`: promotion record and environment model
- `proof`: integrity proof and corruption policy models

## Reproducibility verification

Use `bijux-dev-dag` to verify local artifact reproducibility:

- `cargo run -p bijux-dev-dag -- artifact-verify`

This command checks manifest output hashes against on-disk output files.

## Lifecycle policy

Artifact retention and cleanup policy is defined in:

- `crates/bijux-dag-artifacts/src/retention.rs` (typed policy model)
- `docs/spec/ARTIFACT_RETENTION_POLICY.md` (operator-facing retention and cleanup rules)
