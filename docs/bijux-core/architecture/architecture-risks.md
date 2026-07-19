---
title: Architecture Risks
audience: mixed
type: architecture
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Architecture Risks

Architecture risks track where structure drift can cause expensive debugging,
compatibility loss, or unreliable release outcomes.

## Visual Summary

```mermaid
quadrantChart
    title Architecture Risk Map
    x-axis Low impact --> High impact
    y-axis Easy to detect --> Hard to detect
    quadrant-1 Prioritize mitigation
    quadrant-2 Watch closely
    quadrant-3 Accept or monitor
    quadrant-4 Improve detection
    hidden coupling: [0.79, 0.76]
    state drift: [0.68, 0.72]
    boundary leakage: [0.73, 0.45]
    version mismatch: [0.58, 0.64]
    extension breakage: [0.61, 0.55]
    recovery gaps: [0.82, 0.35]
```

## Key Risk Areas

- runtime and maintainer boundaries becoming coupled
- command behavior changing faster than contracts and docs
- schema evolution without migration or compatibility notes
- multiple orchestration paths diverging from shared gates

## Risk Controls

- ownership checks in maintainer suites
- contract tests for replay/diff and output semantics
- docs-check and link validation on every release candidate
- explicit change and decision records for compatibility-sensitive updates

## Code Anchors

- `crates/bijux-dev/tests/source_layout_guardrails.rs`
- `crates/bijux-dag-app/tests/replay_diff_hardening_contract.rs`
- `makes/docs.mk`
- `mkdocs.yml`

## Next Reads

- [Risk and Exceptions](../governance/risk-and-exceptions.md)
- [Change Management](../operations/change-management.md)
- [Release and Versioning](../operations/release-and-versioning.md)
