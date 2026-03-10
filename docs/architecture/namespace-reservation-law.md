# Namespace Reservation Law

Scope: tasks `421-440`.

Canonical sources:
- `crates/bijux-cli-plugin/src/constants.rs`
- `artifacts/status/reserved_namespace_inventory.json`
- `artifacts/status/namespace_abuse_report.json`

Law:
- reserved namespaces and future product namespaces are immutable compatibility boundaries
- namespace normalization and case-folding must not permit takeover
- reserved-path rejection must remain explicit and machine-readable
