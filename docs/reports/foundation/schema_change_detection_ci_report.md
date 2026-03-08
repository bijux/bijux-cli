# Schema Change Detection CI Report

Schema change detection is enforced through deterministic contract checks in CI.

## CI controls

- Stable hash freeze of schema files via `configs/policy/stable_schema_hashes.json`
- Contract test guard that fails when a frozen schema hash changes
- Changelog presence requirement in `docs/reports/foundation/schema_changelog.md`
- Schema governance workflow presence check: `.github/workflows/schema-governance.yml`

## Enforced by

- `crates/bijux-dev-dag/tests/schema_governance_contracts.rs`
- `crates/bijux-dev-dag/tests/schema_compatibility_guarantees_contracts.rs`
