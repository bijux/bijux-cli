---
title: Risk Register
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Risk Register

Use this page when a change seems acceptable in isolation but may still make
the CLI less trustworthy to users, scripts, or plugin authors.

The risk register exists to keep the highest-impact failure modes visible
instead of letting them hide inside implementation details or optimistic review
language.

## Active Risks

- route and alias drift causing script regressions
- plugin compatibility handling regressions during lifecycle updates
- state-file mutation regressions causing config/history corruption
- documentation drift from command behavior after fast refactors

## Mitigations

- maintain routing laws and golden command-surface tests
- keep plugin lifecycle integration contracts in regular validation loops
- require diagnostics and recovery checks for state write-path changes
- enforce docs shape and review checklist gates for contract changes

## Code Anchors

- `crates/bijux-cli/src/routing/`
- `crates/bijux-cli/src/features/plugins/`
- `crates/bijux-cli/src/features/config/`
- `crates/bijux-cli/tests/`
- `docs/bijux-cli/`

## What Reviewers Should Watch Closely

| Risk | Why it matters |
| --- | --- |
| routing and alias drift | small parser changes can break stable command invocations silently |
| plugin lifecycle regressions | install, inspect, route, or uninstall flows can decay without obvious compile-time signals |
| state mutation regressions | config and history corruption reduces trust in local recovery and diagnostics |
| documentation drift | users follow the handbook into behavior the binary no longer provides |

## Reader Shortcut

If a mitigation exists only on paper and not as a maintained test, gate, or
operational workflow, the risk is still active. Reviewers should treat it as a
real live concern, not historical context.

## Continue Reading

- [Architecture Risks](../architecture/architecture-risks.md)
- [Change Validation](change-validation.md)
- [Failure Recovery](../operations/failure-recovery.md)
