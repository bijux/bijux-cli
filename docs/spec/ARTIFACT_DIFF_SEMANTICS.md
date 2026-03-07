# Artifact Diff Semantics

Artifact diff compares identity and lineage-aware payload evidence:

- `artifact_id`
- producer node fingerprint
- payload `sha256`
- upstream/downstream lineage references

Equivalent artifacts may have distinct provenance context across runs.
