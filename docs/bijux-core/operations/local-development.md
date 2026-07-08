---
title: Local Development
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-04
---

# Local Development

Use this page when you want the shortest honest local workflow for working in
`bijux-core` without inventing ad hoc commands.

The rule is simple: start from documented root entrypoints so local work and CI
keep telling the same story.

## Default Local Path

1. edit code or docs
2. run the narrowest relevant checks
3. inspect diagnostics and generated outputs
4. commit when the evidence matches the change

## Baseline Commands

```bash
make install
cargo check --workspace --all-targets
make docs-check
```

Local runs should use the pinned Rust `1.86.0` toolchain from
`rust-toolchain.toml` so the root commands agree with CI and release jobs.

## Why These Entry Points Matter

- They keep local development aligned with CI and release automation.
- They make review evidence easier to reproduce.
- They reduce the chance that a contributor fixes one path while breaking the
  documented one.

## Local Rule

If a workflow cannot be explained from `Makefile`, `makes/`, or a handbook
page, it is not a healthy repository entrypoint yet.

## Continue Reading

- [Contributor Workflows](contributor-workflows.md)
- [Automation Surfaces](automation-surfaces.md)
- [Maintainer Toolchain Setup](../../bijux-dev/operations/toolchain-setup.md)
