# HPC Replay Scheduler Drift Report

## Scope

Tracks replay mismatch classes caused by HPC scheduler and environment drift.

## Tracked drift classes

- queue/partition/account resource fingerprint drift
- environment module fingerprint drift
- scheduler version drift

## Current policy

- Drift must be reported explicitly.
- Replay fidelity downgrade is required when module fingerprints differ.
