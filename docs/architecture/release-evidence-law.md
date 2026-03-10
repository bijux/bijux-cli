# Release Evidence Law

Release readiness claims are valid only when generated evidence artifacts exist and pass review.

Frozen requirements:

1. Every release candidate includes a generated release evidence bundle.
2. The release bundle must include parity matrix, runtime identity diagnostics, package health diagnostics, plugin hardening report, state corruption hardening report, performance report, and known gaps.
3. Release checklist review must include intentionally different behavior, unresolved partial commands, stale scripts outside `dev cli`, stale docs from docs audit, and weak tests from test audit.
4. Migration notes are generated from artifacts for commands, packaging/install, plugin lifecycle, and state behavior.
5. Release truth report and machine-readable release status manifest are the source of release claims.
6. Public release language must be evidence-only and hype-free.

Evidence sources:

- `artifacts/status/release_evidence_bundle.json`
- `artifacts/status/release_status_manifest.json`
- `artifacts/status/release_truth_report.json`
- `artifacts/status/release_truth_report.txt`
- `artifacts/status/migration_notes_commands.json`
- `artifacts/status/migration_notes_packaging.json`
- `artifacts/status/migration_notes_plugin_lifecycle.json`
- `artifacts/status/migration_notes_state_behavior.json`
