# Installation Guide

Use install diagnostics commands and generated artifacts as source of truth.

Run:

```bash
bijux dev cli runtime-identity --json --no-pretty
bijux dev cli package-health --json --no-pretty
```

Review:
- `artifacts/status/install_source_diagnostics.json`
- `artifacts/status/ambiguous_runtime_diagnostics.json`
- `artifacts/status/install_health_report.json`
- `artifacts/status/remaining_install_ambiguities.json`
