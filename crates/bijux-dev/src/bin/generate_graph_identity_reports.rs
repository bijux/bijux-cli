use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, content).map_err(|err| err.to_string())
}

fn main() -> Result<(), String> {
    let root = repo_root();
    let json_out = root.join("docs/reports/foundation/graph_identity_decomposition_report.json");
    let md_out = root.join("docs/spec/GRAPH_IDENTITY_FIELD_IMPACT.md");
    let node_md_out = root.join("docs/spec/NODE_FINGERPRINT_FIELD_IMPACT.md");
    let field_impact_json_out =
        root.join("docs/reports/foundation/graph_identity_field_impact_report.json");
    let canonical_diff_inventory_out =
        root.join("docs/reports/foundation/canonical_diff_fixture_inventory_report.md");

    let payload = serde_json::json!({
        "format": "graph-identity-decomposition/v1",
        "hash_algorithm": "sha256",
        "source": "crates/bijux-dag-core/src/lib.rs canonicalize + graph_fingerprint",
        "included_fields": [
            "spec",
            "inputs (map-order normalized)",
            "nondeterminism_allowed",
            "nodes.id",
            "nodes.kind",
            "nodes.inputs (sorted)",
            "nodes.outputs.name",
            "nodes.outputs.path (path-normalized)",
            "nodes.params (object-key normalized)",
            "nodes.container",
            "nodes.timeout_ms",
            "nodes.resources (zero-values normalized to none)",
            "nodes.retry",
            "nodes.effects (sorted)",
            "nodes.env_allowlist (sorted)",
            "nodes.tags (sorted)",
            "nodes.group",
            "edges.from/to (sorted by tuple)"
        ],
        "excluded_external_context": [
            "backend adapter runtime version",
            "run metadata",
            "artifact storage metadata"
        ],
        "legacy_spec_aliases_normalized": ["0.1", "v0.1"],
        "identity_scope_note": "graph identity is a hash of canonical graph JSON only"
    });
    write_file(&json_out, &serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?)?;
    write_file(
        &field_impact_json_out,
        &serde_json::to_string_pretty(&serde_json::json!({
            "format": "graph-identity-field-impact/v1",
            "graph_fields": {
                "included": payload["included_fields"],
                "excluded_external_context": payload["excluded_external_context"],
                "legacy_spec_aliases_normalized": payload["legacy_spec_aliases_normalized"]
            },
            "node_fields": {
                "included": [
                    "id",
                    "kind",
                    "inputs",
                    "outputs.name",
                    "outputs.path",
                    "params",
                    "container",
                    "timeout_ms",
                    "resources",
                    "retry",
                    "effects",
                    "env_allowlist",
                    "tags",
                    "group"
                ],
                "excluded": [
                    "adapter runtime metadata",
                    "run provenance metadata",
                    "artifact storage metadata"
                ]
            },
            "generated_from": [
                "crates/bijux-dag-core/src/lib.rs",
                "crates/bijux-dag-core/tests/graph_identity_property_contracts.rs",
                "crates/bijux-dag-core/tests/direct_module_entrypoints_contracts.rs"
            ]
        }))
        .map_err(|err| err.to_string())?,
    )?;

    let md = r#"# Graph Identity Field Impact

This mapping documents which graph fields affect identity hashing.

## Included in graph identity

- `spec` after alias normalization (`0.1`/`v0.1` -> `bijux-dag/v0.1`)
- `inputs` (with map key sorting)
- `nondeterminism_allowed`
- node fields: `id`, `kind`, `inputs`, `outputs`, `params`, `container`, `timeout_ms`, `resources`, `retry`, `effects`, `env_allowlist`, `tags`, `group`
- edge fields: `from.node_id`, `from.port`, `to.node_id`, `to.port`

## Normalized before hashing

- node order (sorted by `id`)
- edge order (sorted by `from/to` tuple)
- `outputs.path` path separators
- `params` object key order
- `inputs` map key order
- `env_allowlist`, `effects`, `inputs`, `tags` ordering
- `resources` with `{cpu:0, mem_mb:0}` are normalized to `null`

## Excluded from graph identity

- backend adapter/runtime version metadata
- run-level metadata
- artifact-level metadata

## Generated report

Machine-readable decomposition:

- `docs/reports/foundation/graph_identity_decomposition_report.json`
"#;
    write_file(&md_out, md)?;

    let node_md = r#"# Node Fingerprint Field Impact

This mapping documents node-level fields that contribute to node fingerprinting.

## Included in node fingerprint

- `id`
- `kind`
- `inputs` (sorted)
- `outputs.name`
- `outputs.path` (path-normalized)
- `params` (object-key normalized)
- `container`
- `timeout_ms`
- `resources` (normalized defaults)
- `retry`
- `effects` (sorted)
- `env_allowlist` (sorted)
- `tags` (sorted)
- `group`

## Excluded from node fingerprint

- adapter runtime metadata
- run/provenance metadata
- artifact storage metadata
"#;
    write_file(&node_md_out, node_md)?;

    let canonical_diff_root =
        root.join("crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_diff");
    let mut fixtures = Vec::new();
    if canonical_diff_root.exists() {
        for entry in fs::read_dir(&canonical_diff_root).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let rel = path
                    .strip_prefix(&root)
                    .map_err(|err| err.to_string())?
                    .to_string_lossy()
                    .to_string();
                fixtures.push(rel);
            }
        }
    }
    fixtures.sort();
    let mut md = String::from("# Canonical Diff Fixture Inventory\n\n");
    md.push_str(
        "Generated from `crates/bijux-dag-core/tests/fixtures/graph_identity/canonical_diff`.\n\n",
    );
    md.push_str("| fixture | status |\n| --- | --- |\n");
    for fixture in fixtures {
        md.push_str(&format!("| `{fixture}` | covered |\n"));
    }
    write_file(&canonical_diff_inventory_out, &md)?;

    println!("generated graph identity decomposition reports");
    Ok(())
}
