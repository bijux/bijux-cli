# Plugin Runtime Law

Plugin runtime is considered stable only when all of the following are true:

1. Scaffolded plugins are installable and checkable for both `python` and `rust` scaffold kinds.
2. Install, uninstall, disable, and enable preserve registry consistency and rollback guarantees.
3. Corrupt registry input is diagnosable and self-repair behavior is explicit.
4. Plugin command failures respect stderr/stdout discipline and stable exit-code mapping.
5. Plugin health evidence is generated in both machine and text forms.

Required evidence artifacts:
- `artifacts/status/plugin_state_report.json`
- `artifacts/status/plugin_health_report.json`
- `artifacts/status/plugin_health_report.txt`
- `artifacts/parity/command_parity_matrix.json`
