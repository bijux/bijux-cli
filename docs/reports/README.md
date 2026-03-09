# Reports policy and index

Audience: maintainers  
Owner: documentation governance  
Status: stable  

## Scope and placement

This directory is not part of the primary documentation navigation. It stores evidence and archival material only.

- `docs/INDEX.md` lists only high-signal entrypoints.
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

- `docs/reports/foundation/FOUNDATION_FINAL_REPORT.md` — [runtime dashboard](./RUNTIME-DASHBOARD.md)
- `docs/reports/foundation/RELEASE_EVIDENCE_REPORT.md` — [evidence dashboard](./EVIDENCE-DASHBOARD.md)
- `docs/reports/foundation/RELEASE_CRITICAL_EVIDENCE_MATRIX.md` — [evidence dashboard](./EVIDENCE-DASHBOARD.md)
- `docs/reports/foundation/EVIDENCE_DASHBOARD.md`
- `docs/reports/foundation/EVIDENCE_CI_EXERCISE_REPORT.md`
- `docs/reports/foundation/REPOSITORY_PROOF_STATEMENT.md`
- `docs/reports/foundation/SYSTEM_HEALTH_DIAGNOSTICS_DOCUMENTATION.md`
- `docs/reports/foundation/RUNTIME_ARCHITECTURE_HEALTH_DASHBOARD.md`
- `docs/reports/foundation/RUNTIME_BOUNDARY_REPORT.md`
- `docs/reports/foundation/RUNTIME_BROAD_SURFACE_INVENTORY.md`
- `docs/reports/foundation/RUNTIME_CONTRACT_BACKING_REPORT.md`
- `docs/reports/foundation/RUNTIME_MODELED_ONLY_SURFACES.md`
- `docs/reports/foundation/RUNTIME_STABLE_VS_EXPERIMENTAL_SURFACE_PAGE.md`
- `docs/reports/foundation/RUNTIME_PUBLIC_API_MAP.md`
- `docs/reports/foundation/KERNEL_API_SURFACE_REPORT.md`
- `docs/reports/foundation/DOCS_ROOT_INVENTORY_REPORT.md`
- `docs/reports/foundation/RUN_HISTORY_SIZE_GROWTH_REPORT.md`
- `docs/reports/foundation/RUN_HISTORY_CORRUPTION_RESILIENCE_REPORT.md`
- `docs/reports/foundation/APP_HOT_PATH_QUALITY_DASHBOARD.md`
- `docs/reports/foundation/CLI_STABILITY_DASHBOARD.md`
- `docs/reports/foundation/INSPECT_DIAGNOSTICS_DASHBOARD.md`
- `docs/reports/foundation/ARCHIVED_LOW_VALUE_FOUNDATION_REPORTS_100.md`

## Dashboard entrypoints

- [Runtime dashboard](./RUNTIME-DASHBOARD.md)
- [Evidence and release dashboard](./EVIDENCE-DASHBOARD.md)
- [Quality and operator dashboard](./QUALITY-DASHBOARD.md)
