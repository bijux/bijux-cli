# Fixture Governance Quick Reference

Policy source: `configs/policy/fixture_family_governance.json`

## Governed families

| Family | Purpose | Owner | Lane | Taxonomy |
| --- | --- | --- | --- | --- |
| `graph` | Deterministic DAG shape, schema, and canonicalization contract coverage. | `core-contracts` | `test` | `product-contract` |
| `run` | Run manifest, run history, and run identity contract coverage. | `app-runtime-contracts` | `test` | `product-contract` |
| `artifact` | Artifact integrity, lineage, and storage behavior contract coverage. | `artifact-contracts` | `test` | `product-contract` |
| `replay` | Replay mismatch and replay fidelity contract coverage. | `app-replay-contracts` | `test` | `product-contract` |
| `diff` | Diff explainability and drift classification contract coverage. | `app-diff-contracts` | `test` | `product-contract` |
| `bundle` | Bundle export/import portability and compatibility contract coverage. | `app-bundle-contracts` | `test` | `product-contract` |
| `capability` | Backend capability query and support matrix contract coverage. | `runtime-capability-contracts` | `test` | `governance-evidence` |
| `benchmark` | Benchmark baseline, threshold, and scenario governance coverage. | `perf-governance` | `test-all` | `performance-governance` |
| `evidence` | Release evidence topology and command-output evidence fixtures. | `dev-governance` | `evidence-all` | `release-governance` |

## Generated reports

- `docs/reports/foundation/GRAPH_FIXTURE_INVENTORY_REPORT.md`
- `docs/reports/foundation/RUN_FIXTURE_INVENTORY_REPORT.md`
- `docs/reports/foundation/ARTIFACT_FIXTURE_INVENTORY_REPORT.md`
- `docs/reports/foundation/replay_fixture_inventory_report.md`
- `docs/reports/foundation/diff_fixture_inventory_report.md`
- `docs/reports/foundation/BUNDLE_FIXTURE_INVENTORY_REPORT.md`
- `docs/reports/foundation/capability_fixture_inventory_report.md`
- `docs/reports/foundation/BENCHMARK_FIXTURE_INVENTORY_REPORT.md`
- `docs/reports/foundation/EVIDENCE_FIXTURE_INVENTORY_REPORT.md`
- `docs/reports/foundation/fixture_governance_missing_owner_report.md`
- `docs/reports/foundation/fixtures_with_no_owning_suite_report.md`
- `docs/reports/foundation/fixtures_with_no_owning_crate_report.md`
- `docs/reports/foundation/UNREFERENCED_FIXTURES_REPORT.md`
- `docs/reports/foundation/duplicate_fixtures_semantic_hash_report.md`
- `docs/reports/foundation/stale_fixture_schema_field_report.md`
