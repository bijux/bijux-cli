# Repo Tree Simplification Status Report (541-560)

## 541-548 inventory and candidate reporting

- inventory: `repo_tree_inventory_541_560_report.md`
- top-25 largest/churn/lowest-covered reports:
  - `TOP_25_LARGEST_FILES_REMAINING_REPORT.md`
  - `TOP_25_HIGHEST_CHURN_FILES_REMAINING_REPORT.md`
  - `TOP_25_LOWEST_COVERED_PRODUCT_PATHS_REPORT.md`
- no-linked-fixture/docs reports:
  - `module_no_linked_fixtures_report.md`
  - `module_no_linked_docs_report.md`
- inline/split candidate reports:
  - `repo_tree_tiny_module_inline_candidates_report.md`
  - `repo_tree_giant_module_split_candidates_report.md`

## 549-554 simplification and boundary controls

- module hygiene policy/gates: `configs/policy/module_hygiene_governance.json`
- dead helper/re-export tracking: `dead_reexports_unused_preludes_report.md`, `duplicate_helper_modules_report.md`
- module ownership and boundary tests anchored in dev-dag contracts.

## 555-560 trend, cleanup page, suite, dashboard, governance, ADR

- shrink trend: `repo_tree_shrink_trend_report.md`
- cleanup candidates: `repo_tree_cleanup_candidates_report.md`
- verification suite: `configs/suites/repo_tree_simplification_verification.json`
- health dashboard: `repo_tree_health_dashboard.md`
- governance rule: `new_large_files_require_split_plan` in module hygiene policy
- ADR: `docs/adr/20260308-REPO-TREE-SHAPE-GOVERNANCE.md`
