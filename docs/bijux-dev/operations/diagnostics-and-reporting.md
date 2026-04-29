---
title: Diagnostics and Reporting
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Diagnostics and Reporting

This page explains how `bijux-dev` turns noisy runtime output into something a
reviewer can act on.

The key step is not generating more text. It is preserving enough signal to
decide what failed, who owns it, and what should happen next.

## Diagnostic Flow

```mermaid
flowchart TD
    signals["test and runtime signals"] --> aggregate["report commands"]
    aggregate --> classify["risk and ownership"]
    classify --> actions["next actions"]
```

## Diagnostic Surfaces

- maintainer verify and suite outputs
- route, coverage, and evidence report generators
- replay and diff hardening reports for DAG behavior
- docs audit and layout validation outputs

## Reporting Rules

- reports must preserve source command and timestamp context
- summary language must match observed evidence
- unresolved failures must not be collapsed into generic success

## First-Response Commands

Run these before deep remediation to lock evidence:

```bash
cargo run -q -p bijux-dev --bin bijux-dev-cli -- quickcheck --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- status --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- parity --format json --no-pretty
make docs-check
```

## Reading Rule

Use this page when the repository is already telling you something is wrong but
the raw output is still too scattered to trust. Move to Incident Response once
the evidence is stable and the remaining question is containment or recovery.

## Code Anchors

- `crates/bijux-dev/src/commands/reporting.rs`
- `crates/bijux-dev/src/report/model.rs`
- `crates/bijux-dev/src/report/write.rs`

## Next Reads

- [Incident Response](incident-response.md)
- [Risk and Exceptions](../../bijux-core/governance/risk-and-exceptions.md)
- [Known Limitations](../governance/known-limitations.md)
