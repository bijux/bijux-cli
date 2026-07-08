# DAG Evidence

`evidence/dag/` is the repository source of truth for executable DAG proof
assets.

Use these directories when a handbook page, README, or test claims that a DAG
behavior is real and repository-backed. The assets here are the concrete runs,
fixtures, and baselines behind those claims.

## Evidence Areas

- `authoring/`: first-run graphs, validation negatives, and instructional
  examples for graph authoring
- `battle/`: trust-critical end-to-end workflows that support release claims
- `cache/`: cache reuse, invalidation, replay, and corruption proof
- `compat/`: supported and unsupported compatibility fixtures for schemas,
  bundles, and run directories
- `fault/`: failure-mode scenarios and the runtime reactions they must produce
- `operator/`: operator-facing inspection and diagnostics scenarios
- `perf/`: approved performance scenarios, fixtures, and enforced baselines
- `compare/`: executable comparison scenarios with explicit non-equivalence
  limits
- `_meta/`: schemas, registries, directory maps, and generated supporting
  metadata

## Governance Sources

- Contract: `evidence/CONTRACT.md`
- Taxonomy: `evidence/taxonomy.md`
- Ownership ledger: `evidence/ownership/evidence_ledger.json`
- Canonical generated registry: `evidence/_meta/registries/evidence_registry.json`
- Shared asset rules: `evidence/_meta/asset_authoring_rules.md`
- Shared metadata schema: `evidence/_meta/schemas/evidence_asset.schema.json`
- Generated directory map: `evidence/_meta/maps/directory_map.json`

Informational inventory markdown files under `evidence/inventory/` are
non-authoritative summaries. The registry and ownership ledger are the
authoritative sources.
