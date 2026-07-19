---
title: Authoring Rules
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Authoring Rules

The make surface should stay readable enough that a maintainer can follow a
target from `make help` to the owning fragment without guesswork.

## Rules

- keep fragments focused on one repository concern
- use artifact-scoped paths by default
- prefer reusable macros for shared guardrails
- keep workflow-facing targets thin and explicit
- document new root entrypoints in this handbook when they become stable

## Reviewer Lens

- can a maintainer find the target from `make help`
- does the target name the concern it owns
- does the fragment placement match the concern

## Next Reads

- [Make Execution Model](make-system-overview.md)
- [Documentation Standard](../governance/documentation-standard.md)
