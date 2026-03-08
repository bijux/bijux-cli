# System Health Reporting Dashboard

## Health status board

| Domain | Status source | Expected signal |
| --- | --- | --- |
| storage health | `run_storage_health` output | `healthy=true` when no anomalies |
| drift diagnostics | `run_drift_dashboard` output | drift classes present and policy linked |
| benchmark signal health | benchmark threshold assertion reports | no release-blocking regression |
| runtime observability | runtime observability reports | telemetry and diagnostics reports present |

## Release gate expectation

System health verification suite must pass before release verification is marked complete.
