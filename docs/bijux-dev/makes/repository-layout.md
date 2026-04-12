---
title: Repository Layout
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Repository Layout

The make fragments mirror the main repository concerns instead of grouping all
commands by tool.

## Layout Intent

- shared bootstrap and clean logic in `_internal.mk`
- shared shell guardrails in `_macro.mk`
- domain-specific workflows in `rust.mk`, `python.mk`, `docs.mk`, `gh.mk`, and
  `dag.mk`

## Layout Rule

Put a target in the fragment that matches the concern it owns. Do not hide a
docs rule in `gh.mk` or a release planner in `docs.mk`.

## Next Reads

- [Package Dispatch](package-dispatch.md)
- [Authoring Rules](authoring-rules.md)
- [Make System Overview](make-system-overview.md)
