---
title: Package Dispatch
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Package Dispatch

The make surface routes work to package-specific tools without forcing
maintainers to remember every underlying command.

## Dispatch Examples

- Rust aggregate targets fan out to workspace `cargo` commands
- Python targets operate on `crates/bijux-cli-python`
- DAG targets dispatch through `cargo run -p bijux-dev --bin bijux-dev-dag --`
- docs targets wrap MkDocs and documentation automation helpers

## Dispatch Rule

The root target should describe the owning package or surface clearly enough
that a maintainer can predict where failures will land.

## Next Reads

- [CI Targets](ci-targets.md)
- [Package Contracts](package-contracts.md)
- [Maintainer Package Destination](../packages/bijux-dev/index.md)
