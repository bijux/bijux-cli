---
title: Current Implemented Capabilities
audience: mixed
type: explanation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-10
---

# Current Implemented Capabilities

This page names the repository-owned capabilities that are implemented today so
other docs do not blur shipped behavior, modeled behavior, and future work.

Use this page when you need a quick authority check for what `bijux-core`
currently proves in code, contracts, tests, and retained evidence.

## Implemented Capability Areas

- local DAG authoring, validation, planning, execution, replay, and retained
  run inspection through the visible `bijux-dag` operator surface
- deterministic run-directory evidence, artifact manifests, cache explanation,
  and replayable run bundles (see [Replay Contract](../../spec/REPLAY_CONTRACT.md))
- built-in runtime adapters for `const`, `shell`, `python`, `http`,
  `file_transform`, and `container`, with governed contract surfaces
- maintainer verification and governance lanes for contracts, docs, release
  evidence, ownership, and repository hygiene
- repository-owned modeling and internal command lanes that are callable for
  proof and maintenance work but are not part of the stable operator promise

## Boundary Rule

Implemented does not mean public, stable, or broadly promised.

- For the current operator promise, use
  [Release Boundary](../../bijux-dag/foundation/release-boundary.md).
- For the limits on those shipped surfaces, use
  [Known Limitations](../../bijux-dag/quality/known-limitations.md).
- For package and crate ownership, use [Package Boundary](package-boundary.md)
  and [Ownership Model](ownership-model.md).

## What This Page Excludes

This page does not catalog speculative future work, modeled platform
expansion or unshipped API promotion lanes. Those belong in the owned product
roadmap
and the DAG release-boundary handbooks.
