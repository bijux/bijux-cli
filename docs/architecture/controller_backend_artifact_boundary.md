# Controller, backend, and artifact boundary

## Local controller
- owns run lifecycle state
- owns scheduler decisions
- persists authoritative manifest and snapshot artifacts

## Remote backend
- receives executable work items from controller
- executes work and returns observations
- does not own authoritative run completion decisions

## Persisted artifacts
- authoritative artifacts: manifest, snapshot, outputs indexes, controller-finalized trace summaries
- observational artifacts: remote backend status events and logs

## Data flow
1. Controller emits work request.
2. Remote backend executes and emits events.
3. Controller reconciles events and updates authoritative run state.
4. Controller finalizes run completion and terminal metadata.

## Integrity rules
- only controller writes terminal run status
- remote events are append-only observations
- reconciliation is deterministic and idempotent
