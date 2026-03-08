use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
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

    println!("generated graph identity decomposition reports");
    Ok(())
}
