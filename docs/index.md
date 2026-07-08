---
title: Bijux Core Documentation
audience: mixed
type: index
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-04
---

# Bijux Core

<!-- bijux-core-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-core/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-core/workflows/release-crates/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml)
[![PyPI Publish](https://github.com/bijux/bijux-core/workflows/release-pypi/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-2%20packages-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-core)
[![Published packages](https://img.shields.io/badge/published%20packages-2-2563EB)](https://github.com/bijux/bijux-core/tree/main/crates)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-dag docs](https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-core` publishes two product families from one repository:

- `bijux`, the root command runtime for apps, plugins, config, diagnostics,
  and interactive workflows
- `bijux-dag`, the local-first DAG system for validation, planning, execution,
  replay, artifact inspection, and verification

The same repository also carries the private maintainer surfaces that keep
those products releasable and testable without splitting the proof from the
implementation.

<div class="bijux-callout"><strong>Start with the surface you care about.</strong>
Repository docs explain shared ownership, publication boundaries, and release
policy. CLI docs own the <code>bijux</code> runtime. DAG docs own
<code>bijux-dag</code>. Maintainer docs own repository gates, diagnostics, and
release proof.</div>

<div class="bijux-panel-grid">
  <div class="bijux-panel"><h3>Repository</h3><p>Use the repository handbook when the question crosses package boundaries, release policy, or shared ownership rules.</p></div>
  <div class="bijux-panel"><h3>CLI</h3><p>Use the CLI handbook for command semantics, runtime behavior, plugin surfaces, REPL behavior, and Python distribution.</p></div>
  <div class="bijux-panel"><h3>DAG</h3><p>Use the DAG handbook for graph compilation, execution, replay, artifacts, and DAG command workflows.</p></div>
  <div class="bijux-panel"><h3>Maintainer</h3><p>Use the maintainer handbook for repository diagnostics, evidence collection, release verification, and policy enforcement.</p></div>
</div>

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="bijux-core/">Open the repository handbook</a>
<a class="md-button" href="bijux-cli/">Open the CLI handbook</a>
<a class="md-button" href="bijux-dag/">Open the DAG handbook</a>
<a class="md-button" href="bijux-dev/">Open the maintainer handbook</a>
</div>

## What Is Public Today

| Surface | Public contract |
| --- | --- |
| `bijux` | the visible `bijux --help` command surface and its Rust and Python distribution paths |
| `bijux-dag` | the visible `bijux-dag --help` surface for local DAG validation, planning, execution, replay, inspection, cache work, and verification |
| maintainer tooling | repository-internal only; documented here for contributors, not shipped as end-user product API |

The DAG handbook also calls out what remains experimental, simulated, or
maintainer-only so local product claims do not drift into platform promises.
When the question is what comes after the current `v0.4.0` local boundary, use
the [Bijux Dag Roadmap](tracking/bijux-dag-roadmap.md).

## Handbook Map

```mermaid
flowchart LR
    home["Bijux Core"] --> repo["Repository handbook"]
    home --> cli["CLI handbook"]
    home --> dag["DAG handbook"]
    home --> dev["Maintainer handbook"]
```

## Start Here

- open [Repository Handbook](bijux-core/index.md) for cross-package
  architecture, release policy, publication boundaries, and shared workflows
- open [CLI Handbook](bijux-cli/index.md) for the `bijux` runtime, official
  app mounting, plugin routing, layered config, and the Python bridge
- open [DAG Handbook](bijux-dag/index.md) for local DAG execution, replay,
  evidence, compatibility, and the supported `bijux-dag` surface
- open [v0.4.0 DAG Release Notes](bijux-dag/operations/v0-4-0-release-notes.md)
  when the question is the current public DAG claim, migration path, examples,
  or validation commands
- open [Bijux Dag Roadmap](tracking/bijux-dag-roadmap.md) when the question is
  which DAG capability lane may come next after the current release boundary
- open [Maintainer Handbook](bijux-dev/index.md) for repository gates, docs
  checks, release verification, diagnostics, and governance operations

## Package Flow

| Handbook | Package destinations | Use it when |
| --- | --- | --- |
| [Repository Handbook](bijux-core/index.md) | [Repository Packages](bijux-core/packages/index.md) | the question is about workspace scope, release policy, or cross-package ownership |
| [CLI Handbook](bijux-cli/index.md) | [`bijux-cli`](bijux-cli/packages/bijux-cli.md), [`bijux-cli-python`](bijux-cli/packages/bijux-cli-python.md) | the issue is command behavior, runtime routing, REPL semantics, or Python distribution |
| [DAG Handbook](bijux-dag/index.md) | [`bijux-dag-core`](bijux-dag/packages/bijux-dag-core.md), [`bijux-dag-runtime`](bijux-dag/packages/bijux-dag-runtime.md), [`bijux-dag-app`](bijux-dag/packages/bijux-dag-app.md), [`bijux-dag-cli`](bijux-dag/packages/bijux-dag-cli.md), [`bijux-dag-artifacts`](bijux-dag/packages/bijux-dag-artifacts.md), [`bijux-dag-testkit`](bijux-dag/packages/bijux-dag-testkit.md) | the issue is graph, execution, replay, artifacts, or DAG command behavior |
| [Maintainer Handbook](bijux-dev/index.md) | [`bijux-dev`](bijux-dev/packages/bijux-dev.md) | the issue is repository diagnostics, release proof, or control-plane automation |

## What Ships Today

| Surface | Public delivery | Summary |
| --- | --- | --- |
| `bijux` | Rust crate, PyPI distribution, release bundles | command runtime for config, history, memory, plugins, mounted apps, REPL, and diagnostics |
| `bijux-dag` | Rust crates and release bundles | deterministic DAG validation, execution, replay, artifact inspection, and evidence-backed comparison |
| `bijux-dev` | repository-internal only | maintainer diagnostics, contracts, evidence, and release workflows |

## Reading Rule

Start with the handbook that owns the user-visible behavior. Move to package
pages only when you need the exact implementation boundary or publication
status.
