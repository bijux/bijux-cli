# Local-only vs remote-coordinated runtime evolution

## Local-only runtime
- one controller process
- one authoritative run-state writer
- direct scheduler to backend calls
- simple failure domains and restart semantics

## Remote-coordinated runtime (future)
- controller plus remote execution backends
- controller remains authoritative writer for run metadata
- backend emits observations; controller reconciles
- additional fault classes: delayed, duplicated, out-of-order, and missing events

## Required invariants before promotion
- terminal state immutability
- idempotent reconciliation
- monotonic per-node sequence handling
- planner and scheduler contract preservation
- storage contract preservation for authoritative artifacts

## Current maturity statement
Remote coordination is simulated for contract tests only and is not a production execution mode.
