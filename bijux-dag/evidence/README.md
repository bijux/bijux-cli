# Evidence

`evidence/` is the repository authority for executable proof assets.

## Subdomain map
- `evidence/authoring`: onboarding evidence split into `minimal`, `patterns`, `negative`, and illustrative `examples`.
- `evidence/battle`: trust-critical end-to-end behavior scenarios.
- `evidence/cache`: cache reuse and corruption correctness scenarios.
- `evidence/compat`: version and format compatibility fixtures.
- `evidence/fault`: fault injection and resilience scenarios.
- `evidence/operator`: operator inspection and diagnostics scenarios.
- `evidence/perf`: performance scenarios and controlled baselines.
- `evidence/compare`: cross-system comparison scenarios and baselines.
- `evidence/_meta`: shared schemas, registries, and generated evidence maps.

## Governance
- Contract: `evidence/CONTRACT.md`
- Taxonomy: `evidence/taxonomy.md`
- Ownership ledger: `evidence/ownership/evidence_ledger.json`
- Canonical generated registry: `evidence/_meta/registries/evidence_registry.json`
- Shared asset rules: `evidence/_meta/asset_authoring_rules.md`
- Shared metadata schema: `evidence/_meta/schemas/evidence_asset.schema.json`
- Generated directory map: `evidence/_meta/maps/directory_map.json`

Informational inventory markdown files under `evidence/inventory/` are non-authoritative summaries. Registry and ledger are authoritative.
