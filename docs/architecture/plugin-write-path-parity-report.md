# Plugin Write-Path Parity Report

Date: 2026-03-09
Scope: stable parity and behavior coverage.

## Implementation status

Rust plugin write paths are implemented in `crates/bijux-cli-plugin`:

- install
- uninstall
- enable
- disable

Registry updates run through atomic save and rollback-aware update flow.

## Transaction and rollback behavior

- Install and uninstall operations use `update_registry` with backup/restore.
- Successful operations clean backup artifacts.
- Failed install/uninstall operations preserve previous registry state.

## Edge-case behavior covered

- Reinstalling a namespace after uninstall works.
- Upgrade/downgrade without explicit uninstall is currently rejected as namespace conflict (documented non-support behavior).
- Installing with reserved namespaces is rejected.
- Installing incompatible manifests is rejected.
- Enabling a plugin recorded as `Broken` is rejected.
- Disabling a missing plugin returns stable not-found error.
- Listing remains stable after failed install attempts.
- Registry persistence across restart reads is verified.

## Python parity status

- `plugins list`: parity shape coverage exists via Python capture-based tests.
- `install/uninstall`: direct Python-vs-Rust command parity is limited by current captured artifact set and route exposure differences.
  - Kept explicit as constrained parity scope rather than inferred parity.

## Status for 321-340

- `321`: complete
- `322`: complete
- `323`: complete
- `324`: complete
- `325`: complete
- `326`: complete
- `327`: complete
- `328`: complete
- `329`: complete
- `330`: complete as explicit non-support without pre-uninstall
- `331`: complete as explicit non-support without pre-uninstall
- `332`: complete
- `333`: complete
- `334`: complete
- `335`: complete
- `336`: complete
- `337`: complete
- `338`: complete for Python-supported `plugins list` parity; install/uninstall parity constrained and documented
- `339`: complete (this report)
- `340`: complete (baseline frozen by parity artifacts and `artifacts/status/plugin_health_report.json`)
