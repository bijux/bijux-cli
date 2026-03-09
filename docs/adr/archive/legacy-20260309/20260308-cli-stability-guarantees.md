# ADR: CLI Stability Guarantees

- Status: accepted
- Date: 2026-03-08

## Context

CLI is the primary operator boundary. Stability requires deterministic command surface,
backward-compatible output behavior, and explicit error-class governance.

## Decision

CLI stability guarantees are:

1. Command surface and aliases remain compatibility-governed.
2. JSON/text outputs remain contract-stable and snapshot-governed.
3. Exit-code and error taxonomy behavior remains explicit and test-enforced.
4. Smoke and malformed-input no-panic coverage remain part of verification suites.

## Enforcement

- Status mapping:
  - `docs/reports/foundation/cli_stability_341_360_status_report.md`
- Inventory/usage/error reports:
  - `docs/reports/foundation/cli_command_inventory_report.md`
  - `docs/reports/foundation/cli_command_usage_heatmap.md`
  - `docs/reports/foundation/cli_error_taxonomy_report.md`
- Stability dashboard:
  - `docs/reports/foundation/cli_stability_dashboard.md`
- Verification suite:
  - `configs/suites/cli_stability_verification.json`

## Consequences

- CLI semantics become a governed product contract.
- CLI behavior changes must preserve compatibility and stable diagnostics guarantees.
