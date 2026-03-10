# Artifact Retention Policy

Keep generated artifacts only when they are consumed by one of these paths:

- CI upload or enforcement gates
- release evidence bundle composition
- maintainer operational workflows

Delete artifacts when all are true:

- no consumer path references them
- they are not required for release evidence
- they do not preserve a unique legal or incident trail

Reject "keep just in case" retention for generated artifacts.
