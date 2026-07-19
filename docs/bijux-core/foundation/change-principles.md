---
title: Change Principles
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Change Principles

The best changes in `bijux-core` make the repository easier to understand after
they land than it was before. That is the standard these principles are meant
to protect.

This repository carries public products, support crates, shared contracts,
published docs, and maintainer automation in one tree. Change discipline is how
those surfaces keep their identities instead of slowly blurring into each other.

## Practical Principles

- prefer stable names over migration-era labels
- change one ownership boundary at a time and document it explicitly
- update docs in the same change that changes behavior or structure
- keep repository rules small enough that product handbooks can stay honest

## What Good Change Looks Like

A strong repository change usually does three things at once:

- it makes the owning surface clearer
- it leaves behind a simpler, more durable steady state
- it gives readers and reviewers enough proof to trust the new state

That can mean code, docs, contracts, tests, or release surfaces all moving in
the same history when the change crosses those boundaries.

## What Usually Requires More Evidence

| Change type | Why the bar is higher |
| --- | --- |
| public contract change | users and downstream automation may rely on it already |
| execution semantics change | behavior can shift even if the command shape stays the same |
| package-boundary change | ownership confusion spreads quickly when crates cross lines |
| release or compatibility rule change | both product families can be affected at once |

## What Usually Should Stay Small

Small changes are still preferred when the surface really is local:

- private implementation cleanup with identical outputs
- targeted docs clarification with no behavior change
- test hardening for one owned surface
- narrowly scoped automation improvements that do not alter release truth

The repository gets brittle when small local changes are forced through large
cross-repo narratives they do not need.

## Repository Smells

- root docs duplicating product detail
- one-off section names that break handbook symmetry without good reason
- automation surfaces that exist in files but not in the handbook
- support crates quietly turning into undocumented product surfaces
- changes that are "technically green" but leave ownership harder to explain

## A Practical Rule Of Thumb

If a change makes a reader ask "who owns this now?" more than before, it
probably moved too many boundaries at once or documented them too weakly.

## What This Page Is Not Saying

- It is not saying docs-only changes never matter.
- It is not replacing crate-level design review.
- It is not encouraging large procedural checklists for ordinary edits.

## Continue Reading

- [Decision Rules](decision-rules.md)
- [Change Management](../operations/change-management.md)
- [Documentation Standards](../governance/documentation-standards.md)
