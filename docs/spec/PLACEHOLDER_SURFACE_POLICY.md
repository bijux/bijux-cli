# Placeholder Surface Policy

## Purpose

Prevent fake completeness by banning placeholder implementations in stable code paths and evidence roots.

## Rules

- Stable source code must not contain `todo!(`, `unimplemented!(`, or panic text that says implementation is missing.
- Public-facing command/output surfaces must not contain placeholder wording unless explicitly allowlisted with an owner and deadline.
- Battle scenarios, release-blocking evidence assets, and operator-facing stable command paths must remain placeholder-free.

## Controlled exceptions

- Object-store runtime boundary message remains explicit until an approved backend contract is implemented.

## Governance artifacts

- Policy: `configs/policy/placeholder_surface_policy.json`
- Inventory report: `docs/reports/foundation/placeholder_inventory_report.md`
- Removal report: `docs/reports/foundation/placeholder_removal_report.md`
- Retention report: `docs/reports/foundation/placeholder_retention_report.md`
- Enforcement test: `crates/bijux-dev-dag/tests/placeholder_surface_contracts.rs`
