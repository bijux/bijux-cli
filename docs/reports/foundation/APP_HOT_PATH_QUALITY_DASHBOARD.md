# App Hot-Path Quality Dashboard

generated_from: route/service tests, contracts, and coverage reports

## Routes

- direct route tests: present for inspect, plan, diagnostics, output selection, surface
- no-panic entrypoint checks: present for inspect/plan/diagnostics
- fast suite: `configs/suites/app_weak_routes_fast.json`

## Services

- replay service direct tests: equivalence, mismatch, imported-run compatibility, downgrade behavior
- replay diff direct tests: grouped mismatch reporting and equivalence summary guarantees

## Renderers and response surfaces

- concise human render assertions:
  - inspect: `concise_explain_human`
  - plan: `concise_plan_lines`
- JSON envelope checks remain under app operator/output contracts

## Linked reports

- Historical linked reports were consolidated for operational review:
  - `docs/reports/foundation/archive/APP_ROUTE_TO_SERVICE_MAPPING.md`
  - `docs/reports/foundation/archive/app_modules_zero_direct_tests_report.md`
  - `docs/reports/foundation/archive/app_modules_below_50_coverage_report.md`
