---
title: Non Hermetic Behaviors
audience: maintainer
type: report
status: active
owner: bijux-dag-maintainers
last_reviewed: 2026-07-06
---

# Non Hermetic Behaviors

This ledger records behavior that remains intentionally outside a full
hermetic-sandbox claim.

## Current behaviors

- local subprocesses still execute on the host operating system
- network denial is a policy contract, not a socket-level firewall
- clock denial is a policy contract, not host clock virtualization
- filesystem authorization protects governed input and output roots, not every
  possible host file read
- container execution semantics do not imply complete isolation

## Maintenance rule

- when a new security-sensitive limitation is discovered, record it here before
  broadening any documentation claim
- when a limitation is removed by a proven implementation change, delete the
  entry in the same change that proves the stronger guarantee
