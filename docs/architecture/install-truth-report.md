# Install Truth Report

Scope: install and packaging truth coverage_ids `361-380`.

## Evidence

- End-to-end channel and path diagnostics: `crates/bijux-cli/src/install/mod.rs`
- Runtime identity and ambiguity diagnostics: `crates/bijux-cli/src/app.rs`
- Maintainer install assumptions command surface: `bijux dev cli package-health`
- Generated artifacts:
  - `artifacts/status/install_source_diagnostics.json`
  - `artifacts/status/ambiguous_runtime_diagnostics.json`
  - `artifacts/status/install_health_report.json`
  - `artifacts/status/install_health_report.txt`
  - `artifacts/status/remaining_install_ambiguities.json`

## Current status

- Install source diagnostics are generated and published.
- Ambiguous runtime diagnostics are generated and published.
- Machine and text install health outputs are generated and published.
- Completion path generation, HOME overrides, XDG-style paths, unwritable state roots, and missing state root bootstrap are covered by tests.
- Maintainer assumptions are exposed through `bijux dev cli package-health`.

## Law

Install and packaging claims are valid only when diagnostics and generated artifacts agree.
