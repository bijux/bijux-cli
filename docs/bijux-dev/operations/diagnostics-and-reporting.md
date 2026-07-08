---
title: Diagnostics and Reporting
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Diagnostics and Reporting

Use this page when the repository is already telling you something is wrong and
you need the shortest path from noisy output to an actionable owner and next
step.

The point of diagnostics is not to generate more text. It is to preserve enough
signal to decide what failed, who owns it, and what should happen next.

## Diagnostic Surfaces

- maintainer verify and suite outputs
- route, coverage, and evidence report generators
- replay and diff hardening reports for DAG behavior
- docs audit and layout validation outputs

## What Good Diagnostics Must Preserve

- reports must preserve source command and timestamp context
- summary language must match observed evidence
- unresolved failures must not be collapsed into generic success

## First Triage Questions

| Question | What to look for |
| --- | --- |
| which command proved the failure? | source command, suite, or report name |
| which surface owns the issue? | CLI, DAG, repository, docs, or release lane |
| is the failure reproducible? | stable machine-readable output and rerunnable command |
| is this a summary or raw evidence problem? | compare overview output with the underlying report or suite |

## First-Response Commands

Run these before deep remediation to lock evidence:

```bash
cargo run -q -p bijux-dev --bin bijux-dev-cli -- quickcheck --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- status --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- parity --format json --no-pretty
make docs-check
```

## Why These Commands Come First

- They freeze the initial evidence before cleanup or retries distort it.
- They identify whether the problem is broad repository health or one owned
  surface.
- They give both human and machine-readable outputs for follow-up work.

## Code Anchors

- `crates/bijux-dev/src/commands/reporting.rs`
- `crates/bijux-dev/src/report/model.rs`
- `crates/bijux-dev/src/report/write.rs`

## Continue Reading

- [Incident Response](incident-response.md)
- [Risk and Exceptions](../../bijux-core/governance/risk-and-exceptions.md)
- [Known Limitations](../governance/known-limitations.md)
