---
title: Operator Inspection Guide
audience: operators
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Operator Inspection Guide

When a run exists already, start from explicit evidence and move from summary
to deeper explanation only as needed.

## Recommended sequence

1. `dag runs list`
2. `dag runs show`
3. `dag runs inspect`
4. `dag runs timeline`
5. `dag runs explain-failure`
6. `dag runs doctor`

Use `dag runs timeline --node <node-id> --event <event-name>` when the question
is about one failure or branch of the run, and add `--since-unix-ms` or
`--until-unix-ms` when narrowing to a precise time window.

## Core inspection principles

- pass `--root` explicitly instead of depending on ambient repository state
- treat `unsupported`, `corrupt`, and `incomplete` as distinct operator
  outcomes
- use `timeline` when timing coherence matters
- prefer `--json` for automation and filtered timeline extraction
- use `doctor` when evidence integrity is the question, not just run status
