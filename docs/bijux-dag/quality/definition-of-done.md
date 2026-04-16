---
title: Definition Of Done
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Definition Of Done

A DAG change is done only when behavior, evidence, and documentation are all
updated and coherent.

## Visual Summary

```mermaid
flowchart TD
  Work[Proposed complete work] --> Code{Code correct and owned?}
  Code -->|No| Rework[Rework]
  Code -->|Yes| Tests{Tests and validation complete?}
  Tests -->|No| Rework
  Tests -->|Yes| Docs{Docs and links updated?}
  Docs -->|No| Rework
  Docs -->|Yes| Review{Review checklist passed?}
  Review -->|No| Rework
  Review -->|Yes| Done[Done]
```

## Done Criteria

- implementation aligns with declared ownership boundaries
- affected tests pass, including contract coverage when applicable
- replay/diff compatibility impact is documented clearly
- docs include updated operator and maintainer guidance
- known limitations and risk posture are updated when needed

## Non-Done Conditions

- tests missing for behavior-changing logic
- docs references stale command names or code anchors
- compatibility-sensitive behavior changed without explicit statement

## Next Reads

- [Change Validation](change-validation.md)
- [Review Checklist](review-checklist.md)
- [Documentation Standards](documentation-standards.md)
