# CLI Surface Stability Policy

Status: accepted
Owner: cli maintainers
Date: 2026-03-09

## Decision
CLI contracts are stable and backward-compatibility aware. Alias/deprecation policy is explicit and machine-verified.

## Consequences
- CLI breaking changes require explicit contract updates.
- Operator command surfaces remain predictable across upgrades.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-CLI-STABILITY-GUARANTEES.md
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
  - `docs/reports/foundation/CLI_STABILITY_341_360_STATUS_REPORT.md`
- Inventory/usage/error reports:
  - `docs/reports/foundation/CLI_COMMAND_INVENTORY_REPORT.md`
  - `docs/reports/foundation/CLI_COMMAND_USAGE_HEATMAP.md`
  - `docs/reports/foundation/CLI_ERROR_TAXONOMY_REPORT.md`
- Stability dashboard:
  - `docs/reports/foundation/CLI_STABILITY_DASHBOARD.md`
- Verification suite:
  - `configs/suites/cli_stability_verification.json`

## Consequences

- CLI semantics become a governed product contract.
- CLI behavior changes must preserve compatibility and stable diagnostics guarantees.
