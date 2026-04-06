---
title: Toolchain Setup
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Toolchain Setup

Maintainer operations require a reproducible local setup that mirrors CI gates
and avoids hidden host-specific behavior.

## Visual Summary

```mermaid
flowchart TD
    clone[clone repository] --> install[install workspace tools]
    install --> build[build workspace]
    build --> verify[run maintainer verify command]
```

## Setup Requirements

- Rust toolchain defined by repository toolchain files
- Python environment and MkDocs dependencies available for docs gates
- `make` targets available for shared workflows

## Baseline Setup Commands

```bash
make install
cargo build --workspace
cargo run -q -p bijux-dev --bin bijux-dev-cli -- verify
make docs-check
```

## Code Anchors

- `Makefile`
- `makes/root.mk`
- `crates/bijux-dev/src/tooling/`

## Next Reads

- [Repository Gates](repository-gates.md)
- [CI and Automation](ci-and-automation.md)
- [Test Policy](../governance/test-policy.md)
