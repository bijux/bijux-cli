# Documentation generation plan

## Source-of-truth documentation

Keep static docs in `docs/` and maintain a manually-updated map in `docs/index.md`.

## Generated artifacts (future work)

1. Export spec examples from tests and keep canonical JSON snapshots.
2. Regenerate API docs when public interfaces change.
3. Add release-time report for doc freshness.

## Current safeguards

- Any new command/suite in `bijux-dev-dag` should have corresponding documentation entry.
- Command and suite contracts should be traceable from docs to tests before merging.
