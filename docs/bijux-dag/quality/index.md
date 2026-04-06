---
title: DAG Quality
audience: maintainers
type: section-index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# DAG Quality

DAG quality defines the proof required for behavior changes, compatibility
claims, and operational trust.

## Visual Summary

```mermaid
flowchart LR
    strategy[test strategy] --> validate[change validation]
    validate --> invariants[invariant protection]
    invariants --> review[review checklist]
    review --> done[definition of done]
```

## Quality Goals

- keep replay and diff semantics stable across change
- require evidence-backed validation before release
- maintain explicit risk and limitation documentation
- align docs with real command and code behavior

## Core Quality Pages

- [Test Strategy](test-strategy.md)
- [Change Validation](change-validation.md)
- [Invariants](invariants.md)
- [Definition of Done](definition-of-done.md)
- [Review Checklist](review-checklist.md)

## Governance Pages

- [Dependency Governance](dependency-governance.md)
- [Risk Register](risk-register.md)
- [Known Limitations](known-limitations.md)
- [Documentation Standards](documentation-standards.md)
