# Plugin Scaffold Minimalism Law

The plugin scaffold output is intentionally minimal and runnable.

## Required files

- Python scaffold: `plugin.manifest.json`, `plugin.py`
- Rust scaffold: `plugin.manifest.json`, `src/lib.rs`

## Forbidden decorative files

Scaffolds must not emit decorative files that do not change runtime behavior, including:

- `README.md`
- `pyproject.toml`
- `Cargo.toml`
- `.gitignore`

## Justification rule

Every scaffolded file must be tracked in `artifacts/status/plugin_scaffold_file_justification.json` with:

- `classification`: `essential`, `helpful`, or `removable`
- `reason`: concrete operational justification

## Change rule

Any scaffold file add/remove/rename requires:

1. Snapshot updates in `crates/bijux-cli/tests/data/golden/cli_surface/plugin_scaffold_*_minimal_files.txt`
2. Passing lifecycle regression in `crates/bijux-cli/tests/integration/cli/plugins/plugin_scaffold_minimal.rs`
3. Regenerated scaffold reports via `bijux dev cli scripts status run --id STATUS-CONTRACT-GENERATE-PLUGIN-SCAFFOLD-REPORTS`
4. Policy pass via `bijux dev cli scripts status run --id STATUS-CONTRACT-ENFORCE-PLUGIN-SCAFFOLD-POLICY`
