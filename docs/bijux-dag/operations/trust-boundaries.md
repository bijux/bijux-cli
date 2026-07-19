---
title: Trust Boundaries
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Trust Boundaries

Use this page when you need the top-level operational boundary for what
`bijux-dag` actually proves versus what it intentionally does not claim.

For the deeper reference wording, open
[Trust Boundaries Reference](trust-boundaries.md). For isolation and
security nuance, open
[Security And Isolation Truth](security-isolation-truth.md).

## Trust what is implemented

- local run execution and artifact production
- manifest, trace, and outputs index verification
- cache proof validation and corruption refusal
- replay and diff classification for locally governed run directories
- rooted write boundaries for run-dir, output, and cache storage helpers

## Do not assume what is not promised

- promoted remote coordination
- distributed consensus for run-state authority
- shell syscall sandboxing, shell network firewalling, or clock virtualization
- VM-grade container isolation

## Operator rule

If a surface is simulated, experimental, or future-facing, treat it as a
modeled contract boundary unless the release boundary explicitly promotes it.
