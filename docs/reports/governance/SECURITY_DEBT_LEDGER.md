---
title: Security Debt Ledger
audience: maintainer
type: report
status: active
owner: bijux-dag-maintainers
last_reviewed: 2026-07-06
---

# Security Debt Ledger

This ledger tracks known security-model debt that should not be hidden behind
broader claims.

## Current debt

- hermetic mode is policy-driven and does not provide full host isolation
- filesystem controls are rooted authorization checks rather than generalized
  sandbox enforcement
- secret detection focuses on common leak patterns and typed handling
  primitives, not exhaustive content inspection

## Maintenance rule

- add a debt entry before normalizing a known security limitation as acceptable
  behavior
- remove an entry only when implementation proof and contract coverage close
  the gap
