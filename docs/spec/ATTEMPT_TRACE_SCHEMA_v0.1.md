# Attempt Trace Schema v0.1

Attempt trace records are distinct from node summary status.

## Required fields

- `node_id` (string)
- `attempt` (integer >= 1)
- `backend_kind` (string)
- `status` (string: success|failed|skipped|cached|cancelled)
- `exit_code` (integer|null)

## Compatibility

- Additive fields allowed in minor updates.
- Existing required fields are stable within `v0.1`.

## Owner

- Runtime execution backend contract.
