# Reports policy and index

Audience: maintainers  
Owner: documentation governance  
Status: stable  

## Scope and placement

This directory is not part of the primary documentation navigation. It stores evidence and archival material only.

- `docs/index.md` lists only high-signal entrypoints.
- `docs/README.md` documents section boundaries and cleanup rules.

## Report surface policy

- Keep only curated, human-readable reports in this folder.
- Generated machine outputs belong in `docs/generated/reports/...`.
- Historical or low-signal snapshots belong in `docs/reports/foundation/archive`.

## Retention

- Curated reports are reviewed quarterly.
- Generated outputs are rotated according to the owning workflow and should not remain indefinitely.
- Archived low-signal snapshots should be moved out once they are no longer referenced by a decision.

## Current curated sections

- `foundation/` now contains only current high-signal summaries and evidence snapshots.
- `foundation/archive/` holds deferred historical report content for traceability.
- `generated/reports/foundation/` stores generated artifacts that should not be treated as primary docs.

## Foundation reports kept in-tree

- `docs/reports/foundation/foundation_final_report.md` — [runtime dashboard](./runtime-dashboard.md)
- `docs/reports/foundation/release_evidence_report.md` — [evidence dashboard](./evidence-dashboard.md)
- `docs/reports/foundation/release_critical_evidence_matrix.md` — [evidence dashboard](./evidence-dashboard.md)
- `docs/reports/foundation/evidence_dashboard.md`
- `docs/reports/foundation/evidence_ci_exercise_report.md`
- `docs/reports/foundation/repository_proof_statement.md`
- `docs/reports/foundation/system_health_diagnostics_documentation.md`
- `docs/reports/foundation/runtime_architecture_health_dashboard.md`
- `docs/reports/foundation/runtime_boundary_report.md`
- `docs/reports/foundation/runtime_broad_surface_inventory.md`
- `docs/reports/foundation/runtime_contract_backing_report.md`
- `docs/reports/foundation/runtime_modeled_only_surfaces.md`
- `docs/reports/foundation/runtime_stable_vs_experimental_surface_page.md`
- `docs/reports/foundation/runtime_public_api_map.md`
- `docs/reports/foundation/kernel_api_surface_report.md`
- `docs/reports/foundation/docs_root_inventory_report.md`
- `docs/reports/foundation/run_history_size_growth_report.md`
- `docs/reports/foundation/run_history_corruption_resilience_report.md`
- `docs/reports/foundation/app_hot_path_quality_dashboard.md`
- `docs/reports/foundation/cli_stability_dashboard.md`
- `docs/reports/foundation/inspect_diagnostics_dashboard.md`
- `docs/reports/foundation/ARCHIVED_LOW_VALUE_FOUNDATION_REPORTS_100.md`

## Dashboard entrypoints

- [Runtime dashboard](./runtime-dashboard.md)
- [Evidence and release dashboard](./evidence-dashboard.md)
- [Quality and operator dashboard](./quality-dashboard.md)
