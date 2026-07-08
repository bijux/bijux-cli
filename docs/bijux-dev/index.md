---
title: Maintainer Handbook
audience: maintainers
type: index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Maintainer Handbook

Use the maintainer handbook when the question is about keeping `bijux-core`
healthy: repository gates, release proof, diagnostics, documentation
generation, and the commands that maintain the repository itself.

This is not the end-user product story. It is the contributor and maintainer
route for the tooling that proves the product story is real.

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="packages/bijux-dev.md">Open the bijux-dev package</a>
<a class="md-button" href="operations/">Open operations</a>
<a class="md-button" href="makes/">Open makes</a>
</div>

## Start Here

| If you need to... | Open this page |
| --- | --- |
| set up or validate maintainer tooling | [Toolchain Setup](operations/toolchain-setup.md) |
| run repository gates before review or release | [Repository Gates](operations/repository-gates.md) |
| investigate failing verification output | [Diagnostics and Reporting](operations/diagnostics-and-reporting.md) |
| handle release or automation incidents | [Incident Response](operations/incident-response.md) |
| change a policy for contracts, dependencies, docs, or tests | [Governance](governance/index.md) |
| understand shared make entrypoints and automation surfaces | [makes](makes/index.md) |

## What Lives Here

- [`bijux-dev`](packages/bijux-dev.md), the repository control plane for
  maintainer automation, diagnostics, contracts, and release verification
- operational guides for gates, evidence collection, incident handling, and
  release work
- governance pages for dependencies, contracts, docs, security, and quality
- the make and GitHub workflow pages that drive repository automation

## Use This Handbook When

- the question is about repository health rather than product behavior
- a release or docs workflow failed and you need the owning maintainer route
- you are changing a repository gate, report, or automation surface
- you need to know which maintainer command or make target owns a validation
  path

## When To Leave This Handbook

- Move to the [CLI Handbook](../bijux-cli/index.md) when the question is about
  `bijux` runtime behavior seen by end users.
- Move to the [DAG Handbook](../bijux-dag/index.md) when the question is about
  DAG authoring, execution, replay, or retained evidence.
- Move to the [Repository Handbook](../bijux-core/index.md) when the question
  is about cross-product package boundaries or shared release policy rather
  than maintainer tooling itself.
