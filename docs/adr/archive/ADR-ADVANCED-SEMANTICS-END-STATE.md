# Advanced Semantics End State

## Status
Accepted

## Context
The runtime boundary, internal surface quarantine, and user-facing stability posture for advanced semantics are captured in the dated architecture record:

- `docs/adr/20260308-ADVANCED-SEMANTICS-RUNTIME-BOUNDARY.md`

Some governance contracts and completion reports reference a stable ADR path that is not date-scoped.

## Decision
This document is the stable ADR anchor for advanced semantics end state references.
It delegates normative architectural details to:

- `docs/adr/20260308-ADVANCED-SEMANTICS-RUNTIME-BOUNDARY.md`

## Consequences
- Contract checks can reference one stable path without coupling to dated filenames.
- The dated ADR remains the canonical change record for architecture history.
