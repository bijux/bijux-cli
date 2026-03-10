# Snapshot Retention Policy

Keep snapshots only when they are tied to live commands and active tests.

Delete snapshots when any of the following is true:

- command path no longer exists
- no active test reads the snapshot
- snapshot duplicates stronger parity evidence already kept

Reject "keep just in case" retention for snapshots.
