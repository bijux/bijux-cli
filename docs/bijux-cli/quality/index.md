---
title: Quality
audience: mixed
type: index
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# CLI Quality

Use this section when the question is not what `bijux` does, but why a reader
or maintainer should trust the behavior claim in front of them.

Quality for `bijux-cli` is not a separate paperwork lane. It is the evidence
that command behavior, tests, compatibility notes, and documentation still tell
one coherent story before a change ships.

## Quality Scope

- test layering and execution expectations
- invariant checks that protect command contracts
- review checklist criteria for safe merges
- documentation and evidence standards
- known limitations and risk-tracking practices

## Code Anchors

- `crates/bijux-cli/tests/`
- `crates/bijux-cli/src/contracts/`
- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `makes/docs.mk`

## How To Read This Section

| If your question is... | Start here |
| --- | --- |
| what proof should exist for this change? | [Change Validation](change-validation.md) |
| what must never drift during refactors? | [Invariants](invariants.md) |
| when is a change truly review-ready? | [Definition of Done](definition-of-done.md) |
| what risks and limitations still remain? | [Risk Register](risk-register.md) and [Known Limitations](known-limitations.md) |

## Pages In This Section

- [Test Strategy](test-strategy.md)
- [Invariants](invariants.md)
- [Review Checklist](review-checklist.md)
- [Documentation Standards](documentation-standards.md)
- [Definition of Done](definition-of-done.md)
- [Dependency Governance](dependency-governance.md)
- [Change Validation](change-validation.md)
- [Known Limitations](known-limitations.md)
- [Risk Register](risk-register.md)

## Reader Shortcut

If a CLI claim changes and this section does not explain what proof moved with
it, the handbook is missing part of the truth. Move to Operations only when
you already know which checks matter and just need to run them.
