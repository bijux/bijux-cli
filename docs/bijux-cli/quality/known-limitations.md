---
title: Known Limitations
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Known Limitations

Use this page when a CLI capability appears to exist but you need the honest
answer about what still has caveats, tradeoffs, or trust boundaries.

Known limitations are not an embarrassment log. They are the line between what
`bijux-cli` proves today and what still requires operator caution, external
dependencies, or explicit expectation management.

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

## What Each Limitation Means In Practice

| Limitation | Operational consequence |
| --- | --- |
| trust-based plugin execution | plugin convenience must not be mistaken for strong isolation |
| external binary delegation | some routes fail because the host is incomplete, not because CLI parsing is wrong |
| large local state | listing, diagnostics, and inspection may degrade before correctness breaks |
| intentionally slow integration suites | not every important regression is visible in the fastest default gate |

## Limitation Rules

- document limitations clearly rather than masking them as temporary noise
- pair every limitation note with the owning code area
- remove limitation entries only when evidence is merged and verified

Latency-related limitation edits should be checked against
`bijux-dev-dag performance-evidence-report` before changing user-facing
expectations.

## Reader Shortcut

If a page claims the CLI supports something broadly but this page still lists a
serious caveat, the limitation wins. The product claim should be read in light
of the operational boundary, not the other way around.

## Continue Reading

- [Risk Register](risk-register.md)
- [Security and Safety](../operations/security-and-safety.md)
- [Architecture Risks](../architecture/architecture-risks.md)
