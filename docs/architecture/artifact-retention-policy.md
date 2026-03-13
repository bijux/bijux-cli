# Retention Policy

Keep generated artifacts, documents, and snapshots only when they have a live
consumer.

## Generated Artifacts

Keep generated artifacts only when they are consumed by one of these paths:

- CI upload or enforcement gates
- release evidence bundle composition
- maintainer operational workflows

Delete generated artifacts when all are true:

- no consumer path references them
- they are not required for release evidence
- they do not preserve a unique legal or incident trail

## Documents

Keep documents when they define durable law, active architecture decisions, or
current operator guidance.

Delete documents when all are true:

- no README, command flow, or contributor flow links them
- content is historical progress reporting rather than enduring policy
- no active runbook depends on them

## Snapshots

Keep snapshots only when they are tied to live commands and active tests.

Delete snapshots when any of the following is true:

- command path no longer exists
- no active test reads the snapshot
- the snapshot duplicates stronger parity evidence already kept elsewhere

Reject "keep just in case" retention for any of these categories.
