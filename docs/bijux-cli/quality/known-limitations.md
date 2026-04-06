---
title: Known Limitations
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Known Limitations

This page tracks current limitations that are important for realistic operational
expectations and accurate release communication.

## Visual Summary

```mermaid
flowchart TB
    limits["known limitations"] --> plugins["plugin trust model limitations"]
    limits --> host["host and packaging boundary limitations"]
    limits --> scale["large state and slow test limitations"]
    limits --> docs["documentation and migration debt limitations"]
```

## Current Limitations

- plugin execution is trust-based and not fully sandboxed
- delegated tool behavior depends on external binary availability
- large local state can increase diagnostic and listing latency
- some integration suites are intentionally slow and run outside fast defaults

## Code Anchors

- `crates/bijux-cli/src/features/plugins/runtime.rs`
- `crates/bijux-cli/src/interface/cli/dispatch/delegation.rs`
- `crates/bijux-cli/src/features/history/operations.rs`
- `crates/bijux-cli/tests/integration/`

## Limitation Rules

- document limitations clearly rather than masking them as temporary noise
- pair every limitation note with the owning code area
- remove limitation entries only when evidence is merged and verified

## Next Reads

- [Risk Register](risk-register.md)
- [Security and Safety](../operations/security-and-safety.md)
- [Architecture Risks](../architecture/architecture-risks.md)
