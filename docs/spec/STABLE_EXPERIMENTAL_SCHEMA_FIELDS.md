# Stable and Experimental Schema Fields

## Stable fields
- DAG: `spec`, `nodes[].id`, `nodes[].command`
- Run manifest: `manifest_version`, `graph_fingerprint`, `status`
- Outputs index: `files[].path`, `files[].sha256`
- Proof bundle: `schema_version`, `proof_id`, `run_id`, `status`

## Experimental fields
- Run manifest: `backend_metadata` (experimental)
- Proof bundle: `signing.signature_format` (experimental)
- Proof bundle: `signing.signature` (experimental)
- Capability matrix: `semantic_portability` (experimental)
