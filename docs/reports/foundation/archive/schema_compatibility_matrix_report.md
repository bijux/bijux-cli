# Schema Compatibility Matrix Report

This report defines the compatibility expectations for schema-facing command outputs and storage payloads.

| Surface | Current version | Backward compatibility | Forward compatibility |
| --- | --- | --- | --- |
| DAG graph schema | `v0.1` | Required for `v0.1` fixtures | Unknown fields rejected unless documented experimental |
| Run manifest schema | `v0.1` | Required for `v0.1` fixtures | Future version rejected with classified error |
| Artifact outputs index schema | `v0.1` | Required for `v0.1` fixtures | Future version rejected with classified error |
| Proof schema | `v0.1` | Required for `v0.1` fixtures | Future version rejected with classified error |
| Diff schema | `run-diff/v0.1` | Required for supported fixtures | Future version rejected with classified error |
| Explain schema | `run-explain-failure/v0.1` | Required for supported fixtures | Future version rejected with classified error |

## Verification links

- `crates/bijux-dev-dag/tests/schema_governance_contracts.rs`
- `crates/bijux-dev-dag/tests/schema_evolution_completion_contracts.rs`
- `crates/bijux-dev-dag/tests/proof_schema_compatibility_contracts.rs`
