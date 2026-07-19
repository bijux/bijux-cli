# Compatibility Evidence

Use compatibility evidence when the claim is about what versions or retained
formats are supported, rejected, or considered corrupt.

## What Lives Here

- `graph_schema/`
- `export_bundle/`
- `run_dir/`

## Classification Rule

- Keep files in `configs/dag/schema/fixtures/` only when they validate schema syntax/shape independent of runtime compatibility semantics.
- Use `evidence/compat/` when a fixture encodes supported versus unsupported compatibility behavior consumed by runtime/app/dev contract tests.

## Boundary

- Compat evidence owns support decision classes: `supported`, `unsupported_newer_version`, `unsupported_older_version`, `corrupt`.
- Compat fixtures are not battle fixtures unless explicitly declared in battle metadata as a consumer.
