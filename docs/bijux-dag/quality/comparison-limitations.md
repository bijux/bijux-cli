---
title: Comparison Limitations
audience: mixed
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Comparison Limitations

Use this page when a comparison result is being interpreted too broadly and you
need the governed limits of what the comparison harness can actually prove.

For the deeper reference wording, open
[Comparison Limitations Reference](reference/comparison-limitations.md). For
the governing contract, open
[Comparison Harness Contract](../../spec/COMPARISON_HARNESS_CONTRACT.md).

The comparison harness is intentionally narrow. It compares scenario behavior
that `bijux-dag` actually measures and documents the limits that remain
outside the comparison boundary.

## Out of scope

- vendor-specific queue internals
- cluster capacity-management economics
- UI ergonomics and IDE integrations
- hardware profile differences
- storage backend implementation details

## Reading limits

- a comparison scenario is not a blanket product ranking
- non-equivalence limits are part of the result, not footnotes
- descriptive scenarios explain context but do not replace factual evidence

## Release posture

Comparison evidence may inform release confidence only through factual
scenarios whose `bijux` side is directly measured and whose limits remain
explicit.
