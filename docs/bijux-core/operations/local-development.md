---
title: Local Development
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-04
---

# Local Development

Local development begins with the owning surface and the smallest documented
command that proves it. Repository-wide gates follow when a change crosses
package, contract, release, or publication boundaries.

The rule is simple: start from documented root entrypoints so local work and CI
keep telling the same story.

## Default Local Path

1. edit code or docs
2. run the narrowest relevant checks
3. inspect diagnostics and generated outputs
4. commit when the evidence matches the change

## Baseline Commands

```bash
make bootstrap
make doctor-rs
cargo check --workspace --all-targets --locked
make docs-check
```

Local runs should use the pinned Rust `1.86.0` toolchain from
`rust-toolchain.toml`. That aligns with the source contract and
repository-owned validation workflows. Synchronized generic CI and release
configuration must be audited separately; the
[Maintainer Toolchain Setup](../../bijux-dev/operations/toolchain-setup.md)
records the current hosted-policy mismatch and its upstream ownership.

## Why These Entry Points Matter

- They keep local development aligned with CI and release automation.
- They make review evidence easier to reproduce.
- They reduce the chance that a contributor fixes one path while breaking the
  documented one.

## Local Rule

If a workflow cannot be explained from `Makefile`, `makes/`, or a handbook
page, it is not a healthy repository entrypoint yet.

## Development References

- [Contributor Workflows](contributor-workflows.md)
- [Automation Surfaces](automation-surfaces.md)
- [Maintainer Toolchain Setup](../../bijux-dev/operations/toolchain-setup.md)
