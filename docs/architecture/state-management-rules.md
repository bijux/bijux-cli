# State Management Rules

State behavior is accepted only when these artifacts and commands agree:

- `bijux dev cli state-audit --json --no-pretty`
- `bijux dev cli state-doctor --json --no-pretty`
- `artifacts/status/state_doctor_report.json`
- `artifacts/status/status_state_corruption_health_report.json`
- `artifacts/parity/state_behavior_parity_matrix.json`

Law:
- malformed-input resilience is required for stateful readers
- rollback or non-corruption proof is required for stateful mutations
- path resolution is shared and must not fork by command surface
