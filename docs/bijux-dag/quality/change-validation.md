---
title: Change Validation
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Change Validation

Every DAG change must ship with evidence that behavior is understood, tested,
and documented.

## Visual Summary

```mermaid
flowchart LR
  Change[Code or docs change] --> Static[Static checks]
  Static --> Unit[Unit and targeted tests]
  Unit --> Contract[Contract and replay/diff checks]
  Contract --> Integration[Integration checks]
  Integration --> Docs[Docs and risk alignment]
  Docs --> Evidence[Review-ready evidence]
```

## Validation Checklist

1. classify impact: interface, runtime, artifact, or docs-only
2. run focused and contract-level tests for impacted area
3. verify replay/diff behavior for compatibility-sensitive changes
4. update docs where behavior or operator guidance changed
5. record remaining risks and mitigations

## Minimum Evidence

- test output tied to changed components
- replay/diff evidence for semantic-impacting updates
- docs updates with stable links and code anchors

## Code Anchors

- `crates/bijux-dag-app/tests/`
- `crates/bijux-dag-core/tests/`
- `crates/bijux-dag-runtime/tests/`

## Next Reads

- [Definition of Done](definition-of-done.md)
- [Review Checklist](review-checklist.md)
- [Release and Versioning](../operations/release-and-versioning.md)
