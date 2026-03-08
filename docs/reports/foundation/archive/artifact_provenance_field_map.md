# Artifact Provenance Field Map

This map records provenance-carrying fields used by artifact identity and run lineage surfaces.

## Run manifest provenance-adjacent fields

- `run_id`
- `graph_fingerprint`
- `tool_version`
- `adapters[]` (`adapter_id`, `adapter_version`, `effects`)
- `policy` (`deny_network`, `deny_env`, `deny_clock`, `clean_env`)
- `run_metadata.submission_source`
- `run_metadata.trigger_source`
- `run_metadata.operator`
- `run_metadata.parent_run_id`
- `run_metadata.source_run_id`

## Output and artifact provenance fields

- `OutputSummary.node_id`
- `OutputSummary.node_fingerprint`
- `OutputSummary.file`
- `OutputSummary.sha256`
- `RunOutputFile.node_id`
- `RunOutputFile.node_fingerprint`
- `RunOutputFile.sha256`
- `RunOutputFile.path`
- `OutputFile.path`
- `OutputFile.sha256`
- `OutputFile.node_id`
- `OutputFile.node_fingerprint`

## Node trace provenance fields

- `NodeTrace.node_id`
- `NodeTrace.fingerprint`
- `NodeTrace.adapter_id`
- `NodeTrace.adapter_version`
- `NodeTrace.replay_provenance.node_action`
- `NodeTrace.replay_provenance.source_run_id`

## Contract references

- `crates/bijux-dag-artifacts/tests/artifact_identity_and_lineage_contracts.rs`
- `crates/bijux-dag-artifacts/tests/run_manifest_identity_contracts.rs`
- `crates/bijux-dag-artifacts/tests/run_manifest_roundtrip_and_retention_contracts.rs`
- `crates/bijux-dag-app/tests/artifact_identity_explain_contract.rs`
