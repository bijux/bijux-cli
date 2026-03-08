# Schema Stability Dashboard

| Signal | Source | Status expectation |
| --- | --- | --- |
| Compatibility fixtures present | `evidence/compat/**` | Required |
| Migration contract determinism | schema migration tests | Required |
| Stable schema hash freeze | `stable_schema_hashes.json` contracts | Required |
| Changelog freshness | `schema_changelog.md` | Required |
| Governance workflow | `.github/workflows/schema-governance.yml` | Required |

This dashboard is intentionally compact and release-oriented.
