---
title: Package Dispatch
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Package Dispatch

Use this page when a root `make` target fails and you need to know which
underlying package, toolchain, or binary actually owns the work.

The make surface exists partly to hide mechanical repetition, but it should not
hide ownership. A maintainer should be able to predict where a failure lands
before opening the implementation.

## Dispatch Map

| Root target family | What it usually dispatches to |
| --- | --- |
| Rust targets | workspace `cargo` commands |
| Python targets | `crates/bijux-cli-python` packaging and release flows |
| DAG targets | `cargo run -p bijux-dev --bin bijux-dev-dag -- ...` |
| docs targets | MkDocs and documentation automation helpers |

## Why Dispatch Exists

- It gives maintainers one predictable shell surface for repeated workflows.
- It keeps package-local complexity out of the root command line.
- It still leaves a clear ownership trail when a command fails.

## Dispatch Rule

The root target should describe the owning package or surface clearly enough
that a maintainer can predict where failures will land.

## What Good Dispatch Looks Like

- a root target name that hints at the owning surface
- output that still reveals the underlying tool or package
- documentation that explains the fan-out without forcing maintainers to read
  every make fragment

## Continue Reading

- [CI Targets](ci-targets.md)
- [Package Contracts](package-contracts.md)
- [Maintainer Package Destination](../packages/bijux-dev.md)
