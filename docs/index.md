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

`bijux-core` is the shared source tree for the `bijux` command runtime, the
`bijux-dag` graph execution system, and the repository-owned support surfaces
that keep both products releasable from one audited workspace.

Start here when you need orientation before reading code: which handbook owns
the question, which package family holds the implementation, and which
repository-level rules sit above the product handbooks.

<div class="bijux-callout"><strong>Start with the surface you care about.</strong>
Repository docs explain workspace rules and release boundaries. CLI docs own
the <code>bijux</code> command product. DAG docs own graph truth, execution,
replay, and artifacts. Maintainer docs own repository gates, diagnostics, and
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

## Handbook Map

```mermaid
flowchart LR
    home["Bijux Core"] --> repo["Repository handbook"]
    home --> cli["CLI handbook"]
    home --> dag["DAG handbook"]
    home --> dev["Maintainer handbook"]
```

## Start Here

- open [Repository Handbook](bijux-core/index.md) when the issue spans CLI,
  DAG, and maintainer ownership or touches repository policy
- open [CLI Handbook](bijux-cli/index.md) for the `bijux` command product and
  its Python bridge
- open [DAG Handbook](bijux-dag/index.md) for graph truth, runtime policy,
  artifacts, replay, and DAG command behavior
- open [Maintainer Handbook](bijux-dev/index.md) for repository gates,
  diagnostics, docs verification, and release proof

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

Start from the handbook that owns the question, then move into its package
pages when you need the exact implementation boundary. If two branches seem to
own the same behavior, verify the split from the
[Repository Handbook](bijux-core/index.md).
