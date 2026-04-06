---
title: Documentation Style Standard
audience: maintainers
type: standard
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Documentation Style Standard

This standard keeps `bijux-core` documentation aligned with the handbook model
already used by other Bijux repositories.

## Required Structure

- One repository-level handbook.
- One handbook per product program.
- One maintainer handbook.
- Identical section taxonomy for product handbooks:
  `foundation`, `architecture`, `interfaces`, `operations`, `quality`.

## Required Language

- Use direct declarative language.
- Prefer short sentences and concrete nouns.
- Avoid ambiguous ownership wording.
- Prefer "owns", "does not own", "proves", and "depends on".

## Required Page Metadata

Every handbook page must include frontmatter keys:

- `title`
- `audience`
- `type`
- `status`
- `owner`
- `last_reviewed`

## Review Expectations

- New pages must be linked from the handbook nav.
- Cross-links must use repository-relative paths.
- Behavior claims must point to implementation or test evidence.
