---
title: Bijux Core Documentation
audience: mixed
type: index
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# Bijux Core

<!-- bijux-core-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-core/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
<!-- bijux-core-badges:generated:end -->

`bijux-core` is one repository with two public products:

- `bijux`, the command runtime for mounted apps, plugins, layered config,
  diagnostics, history, memory, and REPL workflows
- `bijux-dag`, the local-first DAG toolchain for validated graphs, repeatable
  execution, retained evidence, replay, comparison, and verification

The same repository also carries the private maintainer surfaces that release,
audit, and prove those products. Most readers should start with a product
handbook, not the repository or maintainer handbooks.

<div class="bijux-callout"><strong>Start with the thing you want to run.</strong>
Use the CLI handbook for <code>bijux</code>. Use the DAG handbook for
<code>bijux-dag</code>. Use the repository handbook only when the question
crosses products, packages, or release rules.</div>

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

## What Ships Today

| Surface | Delivery | What you can rely on today |
| --- | --- | --- |
| `bijux` | Rust crate, PyPI distribution, release bundles | the visible `bijux --help` runtime, including apps, plugins, layered config, REPL, diagnostics, history, and memory |
| `bijux-dag` | Rust crates and release bundles | the visible `bijux-dag --help` local DAG surface for validate, plan, run, replay, inspect, compare, cache, and verify workflows |
| maintainer tooling | repository-internal only | contributor and release workflows, not end-user product API |

`bijux-dag` is intentionally honest about its current boundary. The stable lane
is local-first. Experimental, simulated, and maintainer-only routes exist in
the repository, but they are not presented here as the default product story.

## Start In The Right Place

| If you want to... | Open this handbook |
| --- | --- |
| run `bijux`, mount apps, work with plugins, or debug runtime behavior | [CLI Handbook](bijux-cli/index.md) |
| author DAGs, run them locally, inspect artifacts, or replay a run | [DAG Handbook](bijux-dag/index.md) |
| understand what the repository publishes, how crates divide work, or how release boundaries are enforced | [Repository Handbook](bijux-core/index.md) |
| work on repository gates, release proof, or documentation and automation pipelines | [Maintainer Handbook](bijux-dev/index.md) |

## Practical Starting Points

- Read [Executable Examples](bijux-dag/interfaces/runnable-examples.md) when you
  want real DAG workflows with expected outputs, not just feature descriptions.
- Read [First-Run Tutorial](bijux-dag/operations/first-run-tutorial.md)
  when you want the shortest route from checkout to a real retained DAG run.
- Read [CLI Runtime Package](bijux-cli/packages/bijux-cli.md) when you already
  know the question belongs to `bijux` and need the crate boundary.
- Read [DAG Release Notes](bijux-dag/operations/v0-4-0-release-notes.md) when
  you want the current release claim in one place.

## How To Read This Site

- Start with a handbook, not a package page.
- Move to package pages when you need the exact crate boundary or Rust import
  lane.
- Move to repository pages when the question crosses more than one product.
- Move to maintainer pages only when you are changing or validating the
  repository itself.

## Documentation Authority

The website contains curated reader guidance. Executable specifications and
generated evidence remain versioned in the repository but are not presented as
product handbook pages. Read the
[documentation system](bijux-core/foundation/documentation-system.md) for the
authority and maintenance rules.

```mermaid
flowchart LR
    question["Reader or maintainer question"]
    handbook["Published handbook<br/>supported behavior and workflow"]
    package["Crate README and internal docs<br/>code ownership and change boundary"]
    contract["Executable specification<br/>enforced invariant"]
    implementation["Source and tests<br/>implemented behavior"]
    evidence["Governed report<br/>observation at a revision"]

    question --> handbook
    handbook -->|implementation detail| package
    handbook -->|normative detail| contract
    package --> implementation
    contract <--> implementation
    implementation -->|governed evaluation| evidence
```

Arrows do not make every document equally authoritative. Handbooks explain the
supported product, crate pages locate implementation ownership, specifications
state enforced behavior, and reports retain observations. A report cannot
override a contract, and an internal package detail cannot widen the public
product promise.

The [v0.4.0 Release Notes](bijux-dag/operations/v0-4-0-release-notes.md) define
the current DAG release. [Future Direction](bijux-dag/foundation/future-direction.md)
is non-binding direction; if it conflicts with the release boundary, the
narrower shipped claim wins.
