# State Model

This document describes the current runtime state model for `bijux` based on executable behavior and generated evidence artifacts.

## Canonical Providers

- Path provider: `crates/bijux-cli/src/app.rs::resolve_state_paths`
- Atomic write provider: `crates/bijux-cli/src/install/io.rs::atomic_write_text`
- Plugin registry write transaction: `crates/bijux-cli-plugin/src/registry.rs::save_registry` and `update_registry`

All writable state surfaces are expected to route through these providers.

## State Files

- Config file: resolved through compatibility discovery (`flags > env > config > defaults`)
- History file: resolved through compatibility discovery and written atomically
- Plugin registry file: resolved from plugins directory and written through temp+rename
- Memory file: resolved from the canonical state root (`<state_root>/.memory.json`) and written atomically

The canonical inventory is generated in:
- `artifacts/status/state_file_inventory.json`

## Behavior and Recovery Contracts

- State behavior coverage: `artifacts/status/unified_state_behavior_report.json`
- Corruption and degraded health evidence: `artifacts/status/unified_state_corruption_report.json`
- Rollback and repair evidence: `artifacts/status/unified_state_rollback_report.json`
- Path resolution evidence: `artifacts/status/unified_state_path_resolution_report.json`
- State doctor snapshots and runtime reports: `artifacts/status/unified_state_doctor_snapshots.json`
- Unified machine-readable audit payload: `artifacts/status/unified_state_audit_payload.json`

## Maintainer Controls

- `bijux dev cli state-audit --format json --no-pretty`
- `bijux dev cli state-doctor --format json --no-pretty`
- `bijux dev cli status --format json --no-pretty` (reports bundle includes unified state artifacts)

## State Law

State behavior is accepted only when these commands and artifacts agree:

- `bijux dev cli state-audit --json --no-pretty`
- `bijux dev cli state-doctor --json --no-pretty`
- `artifacts/status/state_doctor_report.json`
- `artifacts/status/status_state_corruption_health_report.json`
- `artifacts/parity/state_behavior_parity_matrix.json`

Required properties:

- malformed-input resilience for stateful readers
- rollback or non-corruption proof for stateful mutations
- shared path resolution across command surfaces

## Policy Gate

`bijux dev cli scripts status run --id STATUS-CONTRACT-ENFORCE-STATE-LAW-POLICY` fails CI when:

- required state artifacts are missing
- canonical atomic write paths are bypassed
- unified state payload sections are incomplete
- state complexity evidence is missing mutation coverage
- state doctor snapshot evidence is incomplete
