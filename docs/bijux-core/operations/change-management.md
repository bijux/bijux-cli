---
title: Change Management
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Change Management

Change management in `bijux-core` is the discipline of landing a repository
change with one coherent story: what changed, who owns it, what evidence proves
it, and which reader-facing surfaces had to move with it.

That story matters because even a small edit can touch several durable layers
at once: crate behavior, contracts, generated references, retained artifacts,
release framing, and handbook pages.

## The Change Sequence That Scales

1. identify the owning crate, contract, or root surface
2. decide whether the change is crate-local or cross-repository
3. update behavior and the owning explanation in the same work stream
4. run the checks that prove the new state honestly
5. merge only when a reviewer can reconstruct the new steady state without
   guesswork

## Start By Naming The Surface

Before changing anything, state the surface in one sentence. Good examples:

- "This changes the `bijux-dag` release boundary for a visible command lane."
- "This updates a repository handbook page to match the existing package
  boundary."
- "This refreshes a root contract and the maintainer suite that enforces it."

If that sentence is still vague, the change probably needs tighter scope before
implementation starts.

## Distinguish Local Changes From Cross-Surface Changes

### Crate-local change

The owning code, the proof, and the explanation all sit inside one package
family and do not alter a shared contract or release boundary.

### Cross-surface change

The change affects any of the following:

- public command output or machine-readable schemas
- shared contracts under `contracts/`
- retained DAG artifacts or golden references
- root automation, release rules, or published handbook structure

Those changes need stronger explanation because readers and reviewers will
encounter them from more than one direction.

## Impact Classification

Classify the change before choosing evidence or migration work:

| Class | Meaning | Required response |
| --- | --- | --- |
| internal | behavior stays inside one owned implementation surface without changing public or retained meaning | focused owner tests and an explicit scope statement |
| interface | a user-facing, machine-facing, or reader-facing interface changes without necessarily breaking compatibility | interface proof and matching documentation |
| compatibility-sensitive | a retained contract may affect downstream users, stored runs, schemas, or release surfaces | migration analysis, contract validation, and release framing |

## The Rule About Docs

If public meaning changed, the owning documentation should move with the
behavior. In this repository, docs are part of the supported surface, not a
follow-up chore for later.

That does not always mean updating a broad handbook page. It means updating the
right explanatory surface in the same change history that changed the truth.

## Evidence Should Match The Scope

Strong change management avoids both under-explaining and over-proving.

- A docs-only clarification should not pretend to be a runtime release change.
- A compatibility-sensitive change should not rely on prose alone.
- A root contract change should not land without the suite that enforces it.

The question is always the same: what is the smallest honest evidence bundle
for this surface?

For a cross-surface change, the review bundle must identify the ownership
boundary, affected handbook pages, relevant command or test evidence, and the
compatibility impact as `none`, `additive`, or `breaking`.

## Common Failure Modes

- changing a downstream file instead of the owning contract or source
- updating implementation without the reader-facing explanation
- bundling unrelated repository stories into one review
- treating root workflow changes as if they were crate-local edits
- claiming "no behavior change" after a release boundary or schema moved

## Working Rule

Every completed change should leave behind a reviewer-friendly chain from
surface to owner to evidence to explanation.

## Change References

- [Review Expectations](review-expectations.md)
- [Testing and Validation](testing-and-validation.md)
- [Decision Record Policy](../governance/decision-record-policy.md)
- [Risk and Exceptions](../governance/risk-and-exceptions.md)
