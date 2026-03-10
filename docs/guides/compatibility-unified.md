# Compatibility Guide

Use maintainer commands and generated artifacts as the compatibility source.

Run:

```bash
bijux dev cli runtime-identity --json --no-pretty
bijux dev cli parity --json --no-pretty
```

Review:
- `artifacts/status/runtime_unity_report.json`
- `artifacts/parity/binary_vs_python_bridge_parity_report.json`
- `artifacts/parity/command_parity_matrix.json`
