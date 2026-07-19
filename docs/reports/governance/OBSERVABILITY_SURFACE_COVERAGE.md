---
title: Observability Surface Coverage
audience: maintainer
type: report
status: active
owner: bijux-dag-maintainers
last_reviewed: 2026-07-06
---

# Observability Surface Coverage

## Current governed surfaces

- structured runtime events
- reconstructed timeline export
- event-log completeness verification
- sensitive-detail detection
- event sinks for stdout, files, and discard flows

## Current gaps to watch

- observability redaction and coverage must not drift away from the required
  runtime event catalog
- deeper operator-facing docs should continue to distinguish authoritative run
  evidence from derived observability payloads

## Primary proof

- `crates/bijux-dag-runtime/src/diagnostics/runtime/observability.rs`
- `crates/bijux-dag-runtime/tests/observability_contracts.rs`
