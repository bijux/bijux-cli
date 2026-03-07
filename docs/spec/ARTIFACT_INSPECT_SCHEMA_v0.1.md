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

