# Install Ambiguity Law

Install/runtime diagnostics must prefer observable evidence over assumptions.

Frozen requirements:

1. Runtime identity must surface PATH shadowing and ambiguous active-binary selection.
2. Mixed pip/cargo installs, stale wrappers, and active-binary mismatch must be diagnosable.
3. Missing or broken active binary paths must be explicitly reported.
4. Package-health assumptions must be available as machine-readable and text artifacts.
5. Packaging neutrality claims require ambiguity diagnostics artifacts.

Evidence sources:

- `artifacts/status/packaging_ambiguity_report.json`
- `artifacts/status/install_state_assumptions_report.json`
- `artifacts/status/package_health_report.json`
- `artifacts/status/package_health_report.txt`
- `crates/bijux-cli/tests/integration/cli/resilience/install_ambiguity_hardening.rs`
