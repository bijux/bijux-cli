# Compatibility Evidence

Purpose: compatibility truth for graph schema, export bundles, and run directory formats.

Subdirectories:
- `graph_schema/`
- `export_bundle/`
- `run_dir/`

Classification rule:
- Keep files in `configs/dag/schema/fixtures/` only when they validate schema syntax/shape independent of runtime compatibility semantics.
- Use `evidence/compat/` when a fixture encodes supported versus unsupported compatibility behavior consumed by runtime/app/dev contract tests.

Boundary:
- Compat evidence owns support decision classes: `supported`, `unsupported_future`, `unsupported_past`, `corrupt`.
- Compat fixtures are not battle fixtures unless explicitly declared in battle metadata as a consumer.
