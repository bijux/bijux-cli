# Artifact Inspect Schema v0.1

Command:

- `dag artifact-inspect <run_dir> <artifact_id> --json`

Schema:

- `configs/schema/operator/artifact_inspect.schema.json`

Required output fields:

- `artifact_id`
- `artifact_sha256`
- `node_id`
- `node_fingerprint`
- `path`
- `size_bytes`
- `provenance.graph_fingerprint`
- `provenance.run_id`
- `provenance.attempt`
- `lineage.upstream_artifact_ids`
- `lineage.downstream_artifact_ids`
- `identity_explain.artifact_id`
- `identity_explain.composed_from.run_id`
- `identity_explain.composed_from.node_id`
- `identity_explain.composed_from.node_fingerprint`
- `identity_explain.composed_from.artifact_sha256`
- `identity_explain.composed_from.path`
