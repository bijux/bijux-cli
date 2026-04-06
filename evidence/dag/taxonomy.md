# Evidence Taxonomy

## Evidence classes
- `authoring`
- `battle`
- `cache`
- `compat`
- `fault`
- `operator`
- `perf`
- `compare`

## Asset kinds
- `scenario`: executable flow or behavior proof.
- `fixture`: structured input for a deterministic behavioral assertion.
- `baseline`: approved expectation used by performance or comparison checks.
- `catalog`: index that binds stable IDs to evidence assets.
- `report`: derived artifact from evidence consumers.

## Stability fields
- `version`: schema or family version for readers and migration tooling.
- `implementation_status`: `implemented` or `simulated`.
- `release_blocking`: whether the asset is required for release trust gates.

## Trust mapping
`battle` assets must declare `trust_properties_protected` and include at least one trust property from policy.

## Shared rule references
- Metadata schema: `evidence/_meta/schemas/evidence_asset.schema.json`
- Naming and structure policy: `evidence/_meta/asset_authoring_rules.md`
