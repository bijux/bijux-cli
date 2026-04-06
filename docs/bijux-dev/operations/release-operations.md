---
title: Release Operations
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Release Operations

Release operations coordinate verification, compatibility review, tagging, and
publishing across programs.

## Visual Summary

```mermaid
flowchart LR
    verify[verify candidate commit] --> review[compatibility and docs review]
    review --> tag[tag approved release]
    tag --> publish[publish artifacts]
    publish --> monitor[post-release monitoring]
```

## Release Workflow Rules

- only tag commits with green required gates
- include compatibility notes for CLI and DAG changes
- ensure docs navigation and links are valid before publishing
- verify post-release health and rollback readiness

## Standard Commands

```bash
cargo run -q -p bijux-dev --bin bijux-dev-cli -- verify
cargo run -q -p bijux-dev --bin bijux-dev-cli -- release
make docs-check
```

## Code Anchors

- `crates/bijux-dev/src/commands/cli_release_command.rs`
- `crates/bijux-dev/src/suites/release.rs`
- `.github/workflows/`

## Next Reads

- [Core Release and Versioning](../../bijux-core/governance/release-and-versioning.md)
- [Contract Governance](../governance/contract-governance.md)
- [Known Limitations](../governance/known-limitations.md)
