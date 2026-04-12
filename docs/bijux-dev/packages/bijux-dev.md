---
title: bijux-dev Package
audience: maintainers
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# bijux-dev

`bijux-dev` is the unified maintainer and governance control plane for the
`bijux-core` workspace. It owns repository diagnostics, evidence gathering,
release verification, and control-plane command surfaces that do not belong in
end-user packages.

Use this page when the question is about repository health, release proof,
automation, or maintainer-only workflows.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| maintainer automation | `bijux-dev-cli` flows, diagnostics, inventories, support tooling |
| governance control plane | repository evidence, suite orchestration, release verification, report assembly |
| repository policy support | docs gates, contract enforcement, and maintainer-only reporting surfaces |
| boundary | does not own end-user CLI runtime semantics or DAG execution semantics |

## Source Layout

- `crates/bijux-dev/src/maintainer`
- `crates/bijux-dev/src/commands`
- `crates/bijux-dev/src/suites`
- `crates/bijux-dev/src/repo`
- `crates/bijux-dev/src/report`
- `crates/bijux-dev/src/bin`
- `crates/bijux-dev/tests`

## Open Next

- open the [Maintainer Handbook](../../index.md) for operations and governance guidance
- open the [Repository Handbook](../../bijux-core/index.md) when a maintainer question touches cross-program ownership
- open [CLI Handbook](../../bijux-cli/index.md) or [DAG Handbook](../../bijux-dag/index.md) when the issue belongs to product behavior rather than governance tooling

## Code Anchors

- `crates/bijux-dev/README.md`
- `crates/bijux-dev/CONTRACT.md`
- `crates/bijux-dev/src/bin`
- `crates/bijux-dev/tests`

## Review Lens

- maintainer-only behavior should stay out of user-facing packages
- release proof and evidence gathering should remain inspectable and reproducible
- repository policy support should point back to handbook guidance instead of inventing parallel rules
