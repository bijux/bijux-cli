---
title: Concurrency Flake Ledger
audience: maintainer
type: report
status: active
owner: bijux-dag-maintainers
last_reviewed: 2026-07-19
---

# Concurrency Flake Ledger

This ledger records known flaky or timing-sensitive concurrency evidence.

## Current status

No active concurrency flakes are recorded at this time.

## Monitoring expectations

- new nondeterministic failures in `crates/bijux-dag-runtime/tests/concurrency_contracts.rs`
  must be recorded here before they are normalized as intermittent CI noise
- any flake entry must name the failing test, triggering environment, observed
  symptom, and removal condition
- once a flake is fixed, remove the entry in the same change that proves the
  fix
