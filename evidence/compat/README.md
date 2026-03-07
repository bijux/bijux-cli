# Compatibility Evidence

Purpose: compatibility truth for graph schema, export bundles, and run directory formats.

Subdirectories:
- `graph_schema/`
- `export_bundle/`
- `run_dir/`

Classification rule:
- Keep files in `configs/schema/fixtures/` only when they validate schema syntax/shape independent of runtime compatibility semantics.
- Use `evidence/compat/` when a fixture encodes supported versus unsupported compatibility behavior consumed by runtime/app/dev contract tests.
