# Run state machine

States:
- `queued`
- `ready`
- `running`
- `succeeded`
- `failed`
- `cached`
- `skipped`
- `cancelled`

Legal transitions:
- `queued -> ready`
- `ready -> running`
- `running -> succeeded|failed|cached|skipped|cancelled`
- `ready -> cancelled`
- `queued -> cancelled`

Illegal transitions are rejected by contract tests.
