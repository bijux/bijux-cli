# Run recovery and resilience contracts

This contract defines run orchestration and recovery behavior for pause/resume, interruption handling, metadata repair, and fault simulation.

## Pause and operator control

- `RunPausePolicy` defines deterministic pause handling for queued, ready, and dispatch behavior.
- `NodeControlMode` supports explicit operator node pause/block records.
- Running nodes can be preserved while queue/dispatch is frozen based on `RunPauseMode`.

## Restart and scheduler recovery

- `PersistedRunSnapshotRef` captures persisted run snapshots used on process restart.
- `SchedulerRecoveryRule` maps orphaned node states to deterministic actions:
  - reattach
  - requeue
  - mark failed
  - quarantine

## Heartbeats, stuck detection, and interruption classes

- `NodeHeartbeatPolicy` and `StuckRunPolicy` define principled liveness thresholds.
- `InterruptionClass` distinguishes:
  - clean shutdown
  - process crash
  - worker loss
  - backend loss
- `ResumePolicy` defines explicit resume choices: reattach, verify, rerun incomplete, fail-safe stop.

## Operator retry, checkpoint resume, and branch-local recovery

- `OperatorRetryPolicy` captures audited manual retry constraints.
- `ManualInterventionRecord` stores first-class human interventions.
- `CheckpointResumeContract` records checkpoint-based node resume capability.
- `BranchRecoveryMode` enables either fail-fast or continue-healthy-branches execution.

## Degraded mode and resilience verification

- `DegradedExecutionPolicy` supports execution when optional services are unavailable.
- Recovery fault simulation contracts:
  - `RecoveryFaultInjection`
  - `RecoverySimulationScenario`
  - `RecoveryAcceptanceSuite`
- Fixture paths:
  - `evidence/perf/fixtures/recovery/power_loss_restart.json`
  - `evidence/perf/fixtures/recovery/acceptance_suite.json`

## Metadata repair, consistency checks, and quarantine

- `validate_and_repair_run_metadata` defines repair behavior for missing manifest/index files.
- `check_run_consistency` validates node state, artifact state, and run summary consistency.
- `RunQuarantineRecord` captures suspicious or inconsistent run evidence.

## CLI control-plane paths

`bijux-dev-dag` exposes:

- `dag repair-run --run-dir <path> [--apply]`
- `dag simulate-recovery --scenario <fixture.json>`
- `dag recovery-accept --suite <fixture.json>`
