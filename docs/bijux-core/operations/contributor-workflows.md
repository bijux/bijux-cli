---
title: Contributor Workflows
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Contributor Workflows

This page explains the normal path from a question or issue to a reviewable
change.

The goal is not to force one ceremony for every edit. It is to make the default
workflow obvious enough that contributors do not need private repository
customs to work effectively.

## Workflow Map

```mermaid
flowchart LR
    contributor["contributor"] --> identify["identify owning handbook and package"]
    identify --> change["implement code and docs change"]
    change --> validate["run root and package validation"]
    validate --> evidence["assemble reviewable evidence"]
    evidence --> review["submit for review"]
    review --> iterate["adjust and retry when needed"]
    iterate --> change
```

## Standard Flow

1. identify the owning handbook and package family
2. make the change in the owning code and docs
3. run the relevant root and package validation commands
4. include reviewable evidence in the change set

## Workflow Rule

Repository work should reduce manual interpretation. If a reviewer needs tribal
knowledge to understand a root workflow, the docs are incomplete.

## Reading Rule

Use this page when the work spans more than one file and the first question is
how to move through the repository in the expected order.

## Next Reads

- [Local Development](local-development.md)
- [Review Expectations](review-expectations.md)
- [Change Management](change-management.md)
