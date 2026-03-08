# Artifact Identity to Provenance Mapping

This mapping defines which artifact fields represent content identity and which represent provenance context.

| Field | Category | Meaning |
| --- | --- | --- |
| `artifact_id` | identity | Logical artifact selector (`node_id:file_name`) used by operator surfaces. |
| `artifact_sha256` | identity | Content-addressed digest over payload bytes. |
| `node_fingerprint` | identity | Node execution identity that produced the artifact payload. |
| `path` | identity | Normalized relative payload location within the run directory. |
| `provenance.run_id` | provenance | Finalized run directory identity that produced/imported the artifact. |
| `provenance.graph_fingerprint` | provenance | Graph-level identity context for provenance traversal and replay checks. |
| `provenance.attempt` | provenance | Attempt lineage marker for retries or replay-derived materialization. |
| `lineage.upstream_artifact_ids` | provenance | Immediate upstream artifact dependencies for explain/trace flows. |
| `lineage.downstream_artifact_ids` | provenance | Immediate downstream artifacts that depend on this artifact. |

Boundary rule:
- Identity fields are used for stable equality and cache/replay compatibility checks.
- Provenance fields are used for ancestry, explainability, and operator audit flows.
- Provenance changes must not be interpreted as payload identity changes unless identity fields also change.
