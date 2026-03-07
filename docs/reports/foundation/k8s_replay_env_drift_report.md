# Kubernetes Replay Environment Drift Report

## Scope

Document known replay drift classes when source execution metadata and target environment diverge.

## Tracked drift classes

- container image digest drift
- namespace policy drift
- required secret/config availability drift
- scheduler capability declaration drift

## Current status

- Drift classes are contract-tracked and must be surfaced explicitly in replay diagnostics.
- Silent replay downgrade is disallowed.
