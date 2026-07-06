---
title: Architecture Review Checklist
audience: maintainer
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Architecture Review Checklist

## Review checks

- repository architecture report is present
- runtime module ownership report is present
- artifact contract report is present
- dependency direction and crate boundaries remain documented
- high-risk surfaces are backed by contract tests and maintainer suites

## Exit rule

Architecture review is incomplete when any required report or governing spec is
missing, stale, or contradicted by executable checks.
