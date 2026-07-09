---
title: Spec To Code And Test Ownership
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Spec To Code And Test Ownership

Specs in `bijux-core` are only useful if a reviewer can trace them in both
directions:

- from the spec to the owning code and enforcing tests
- from the code back to the spec or contract that explains the behavior

If either direction is missing, the repository is relying on memory instead of
ownership.

## What Ownership Means Here

A trustworthy spec has three visible links:

1. an owning code surface
2. an enforcing test or contract suite
3. a reader-facing explanation in the right handbook or package page

The spec is the statement of intent, not the whole proof story by itself.

## What Readers Should Be Able To Trace

| If the page claims... | Readers should be able to find... |
| --- | --- |
| command behavior | the owning CLI or DAG crate plus the tests that exercise the visible output |
| schema or machine-readable compatibility | the contract file and the suites that reject drift |
| replay, artifact, or runtime behavior | the owning DAG crate, retained artifact surface, and the integration or contract tests that verify the behavior |
| repository policy or release rules | the root contract and the maintainer suite that enforces it |

## Rules For Maintaining That Traceability

- spec pages should identify the owning crate, package family, or root contract
- contract pages should make it clear which suites enforce them
- handbook claims should not replace the executable or machine-readable source
- reports may summarize proof, but they are not the authority for behavior

## A Practical Review Loop

When reviewing a spec-backed change, ask these questions in order:

1. Which crate or root surface owns the behavior?
2. Which test, snapshot, or contract suite fails if the behavior drifts?
3. Which public or maintainer-facing page explains the steady state correctly?

If the answer to any one of those questions is missing, the change is still
under-specified.

## What This Rule Prevents

- specs that survive after their owning implementation moved
- generated reports being treated as if they were canonical specs
- public docs promising behavior that no executable suite actually proves
- tests passing while the written contract silently points somewhere else

## Next Reads

- [Documentation Standards](documentation-standards.md)
- [Documentation Governance Alignment](documentation-governance-alignment.md)
- [Testing and Validation](testing-and-validation.md)
