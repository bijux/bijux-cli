# ADR: Runtime Adapter and Registry End State

- Date: 2026-03-08
- Status: Accepted

## Context
Runtime adapter and backend capability behavior must stay deterministic and evidence-backed across releases. Drift in adapter registration, capability query output, or contract validation can silently break replay and portability guarantees.

## Decision
- Keep adapter identity and kind registration deterministic and strict.
- Keep backend capability docs generated from command-aligned outputs only.
- Keep claim-to-evidence mapping generated and release-gated.
- Require direct runtime or release-contract test evidence for each shipped adapter/backend surface.

## Consequences
- Adapter/registry behavior remains predictable for operations and replay.
- Backend capability pages cannot diverge from executable surfaces.
- Release checks fail early when shipped adapter coverage or evidence links regress.
