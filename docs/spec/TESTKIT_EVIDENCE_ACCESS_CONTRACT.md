# Testkit Evidence Access Contract

## Purpose

`bijux-dag-testkit` provides the shared read-only access boundary for governed evidence assets in tests.

## Access helpers

- `evidence_registry_path(workspace_root)` returns the canonical registry location.
- `load_evidence_registry(workspace_root)` loads the registry and panics on malformed state.
- `load_evidence_registry_checked(workspace_root)` returns actionable read/parse errors with the registry path.
- `resolve_evidence_asset_by_id(registry, id)` resolves one asset and panics if missing.
- `resolve_evidence_asset_by_id_checked(registry, id)` returns actionable diagnostics for missing ids.
- `evidence_asset_ids(registry)` returns stable sorted asset IDs for reload-drift checks.

## Rules

- Helpers are read-only. They never mutate files under `evidence/`.
- Tests must resolve canonical assets by id through these helpers instead of hand-wired filesystem crawling.
- Missing assets must return diagnostics that include:
  - the missing asset id
  - a next-step hint to verify ownership and consumer mapping

## Consumer expectations

- Crate tests can keep implementation-local fixtures, but canonical scenario truth stays under `evidence/`.
- Registry reload operations must preserve the set of asset ids unless evidence sources change.
