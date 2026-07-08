---
title: Change Principles
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Change Principles

Use this page when you need the reader-facing version of the repository change
bar: what kinds of changes need more proof, and what kinds should stay small
and obvious?

Repository changes should make ownership clearer, not blurrier. The point is
not process theater. The point is to keep public products, private support
code, and repository rules from drifting into each other unnoticed.

## Practical Principles

- prefer stable names over migration-era labels
- change one ownership boundary at a time and document it explicitly
- update docs in the same change that changes behavior or structure
- keep repository rules small enough that product handbooks can stay honest

## What Usually Requires More Evidence

| Change type | Why the bar is higher |
| --- | --- |
| public contract change | users and downstream automation may rely on it already |
| execution semantics change | behavior can shift even if the command shape stays the same |
| package-boundary change | ownership confusion spreads quickly when crates cross lines |
| release or compatibility rule change | both product families can be affected at once |

## Repository Smells

- root docs duplicating product detail
- one-off section names that break handbook symmetry without good reason
- automation surfaces that exist in files but not in the handbook

## What This Page Is Not Saying

- It is not saying docs-only changes never matter.
- It is not replacing crate-level design review.
- It is not encouraging large procedural checklists for ordinary edits.

## Continue Reading

- [Decision Rules](decision-rules.md)
- [Change Management](../operations/change-management.md)
- [Documentation Standards](../governance/documentation-standards.md)
