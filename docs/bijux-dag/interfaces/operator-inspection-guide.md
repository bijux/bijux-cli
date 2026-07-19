---
title: Operator Inspection Guide
audience: operators
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Operator Inspection Guide

When a run already exists, start from explicit evidence and move from summary
to deeper explanation only as needed.

## Recommended Sequence

1. `bijux-dag runs list`
2. `bijux-dag runs show`
3. `bijux-dag runs inspect`
4. `bijux-dag runs timeline`
5. `bijux-dag runs scheduler-checkpoint`
6. `bijux-dag runs explain-failure`
7. `bijux-dag runs doctor`

## Reading Rules

- pass `--root` explicitly instead of depending on ambient repository state
- treat `unsupported`, `corrupt`, and `incomplete` as distinct operator states
- use `timeline` when timing coherence matters
- use `scheduler-checkpoint` when scheduler state, not just event order, is the
  question
- prefer `--json` for automation and filtered extraction
- use `doctor` when evidence integrity is the question, not just run status

## Exact Retained Paths

Use [Run Evidence Layout](run-evidence-layout.md) when you need the
exact retained file path for manifests, traces, indexes, cache records, or
promotions before opening a run by hand.

## Detailed Walkthrough

Use [Guide: Operator Inspection](operator-inspection-guide.md) for the
longer operational walkthrough.
