# Scheduler state transitions

## Scheduler work item lifecycle

- `ready` -> `scheduled`
- `scheduled` -> `completed_success`
- `scheduled` -> `completed_cached`
- `scheduled` -> `completed_skipped`
- `scheduled` -> `completed_failed`
- `scheduled` -> `retry_queued`
- `retry_queued` -> `retry_requeued`
- `retry_requeued` -> `scheduled`

## Transition constraints

- a downstream node may transition to `ready` at most once per run attempt lineage
- no node may exist in both retry queue and ready queue simultaneously
- cancellation blocks new scheduling decisions
- scheduler timeout returns timeout scheduling outcome without mutating failure classification

## Terminal scheduling outcomes

- `completed_success`
- `completed_cached`
- `completed_skipped`
- `completed_failed`
