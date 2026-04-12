---
title: Local Development
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Local Development

Repository-local development should start from root entrypoints so local work
and CI stay aligned.

## Baseline Commands

```bash
make install
cargo check --workspace --all-targets
make docs-check
```

## Local Rule

If a workflow cannot be explained from `Makefile`, `makes/`, or a handbook
page, it is not a healthy repository entrypoint yet.

## Next Reads

- [Contributor Workflows](contributor-workflows.md)
- [Automation Surfaces](automation-surfaces.md)
- [Maintainer Toolchain Setup](../../bijux-dev/operations/toolchain-setup.md)
