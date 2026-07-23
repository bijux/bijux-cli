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

Both products are built for operations that must remain explainable under
automation. `bijux` resolves a command to one owned runtime and preserves its
stream and exit semantics. `bijux-dag` turns a graph into a validated plan,
executes it through an explicit backend, and retains the evidence needed to
inspect, compare, or replay the result.

The same repository carries private verification surfaces that test release
claims against code, schemas, fixtures, and generated evidence. Those
maintainer tools validate the products; they are not hidden product APIs.

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

## Two Execution Paths

```mermaid
flowchart TB
    operator["operator or automation"]

    subgraph cli_path["Command runtime"]
        cli_input["CLI, REPL, or Python launcher"]
        cli_route["normalize config and resolve route"]
        cli_run["built-in, mounted app, or plugin execution"]
        cli_result["stdout · stderr · exit status"]
    end

    subgraph dag_path["Workflow runtime"]
        dag_input["graph source"]
        dag_plan["validate, canonicalize, and plan"]
        dag_run["backend execution"]
        dag_result["run directory · traces · artifacts · identity"]
    end

    operator --> cli_input --> cli_route --> cli_run --> cli_result
    operator --> dag_input --> dag_plan --> dag_run --> dag_result
```

The paths deliberately end differently. A CLI command returns process-facing
streams and status. A DAG run also leaves an integrity-bearing record because
replay, comparison, and post-run verification depend on durable evidence.

## Trust Properties

| Property | What the repository preserves | Where to verify it |
| --- | --- | --- |
| owned command routing | aliases normalize to canonical routes; delegated processes retain native streams and exit status | [CLI execution model](bijux-cli/architecture/execution-model.md) |
| explicit configuration | layered values can be traced to their source and secret-like fields are redacted by default | [CLI configuration guide](bijux-cli/interfaces/config-guide.md) |
| deterministic graph meaning | canonical graph and plan identities are separate from runtime and artifact identity | [DAG reproducibility model](bijux-dag/interfaces/reproducibility-model.md) |
| failure evidence | failed, skipped, blocked, cancelled, cached, and successful work remain distinguishable | [DAG failure recovery](bijux-dag/operations/failure-recovery.md) |
| honest isolation | enforced checks are separated from host, container, scheduler, and cluster assumptions | [Execution security](bijux-dag/operations/security-isolation-truth.md) |
| release traceability | publication boundaries and required evidence are machine-readable and contract-tested | [Repository release operations](bijux-core/operations/release-and-versioning.md) |

## Start In The Right Place

| If you want to... | Open this handbook |
| --- | --- |
| run `bijux`, mount apps, work with plugins, or debug runtime behavior | [CLI Handbook](bijux-cli/index.md) |
| author DAGs, run them locally, inspect artifacts, or replay a run | [DAG Handbook](bijux-dag/index.md) |
| understand what the repository publishes, how crates divide work, or how release boundaries are enforced | [Repository Handbook](bijux-core/index.md) |
| work on repository gates, release proof, or documentation and automation pipelines | [Maintainer Handbook](bijux-dev/index.md) |

## Practical Starting Points

- To automate `bijux`, begin with the
  [CLI surface](bijux-cli/interfaces/cli-surface.md), then use the
  [configuration guide](bijux-cli/interfaces/config-guide.md) and
  [diagnostics guide](bijux-cli/operations/diagnostics-guide.md).
- To execute a real graph, follow the
  [first-run tutorial](bijux-dag/operations/first-run-tutorial.md), then read
  the [run evidence layout](bijux-dag/interfaces/run-evidence-layout.md).
- To diagnose a failed run, preserve it and follow
  [observability and diagnostics](bijux-dag/operations/observability-and-diagnostics.md)
  before attempting recovery.
- To evaluate a release or repository change, start with
  [testing and validation](bijux-core/operations/testing-and-validation.md)
  and the [maintainer gate map](bijux-dev/operations/repository-gates.md).

## From Outcome To Trust

```mermaid
flowchart LR
    request["command or graph request"]
    contract["validated route, schema,<br/>graph, or policy"]
    execution["owned execution"]
    outcome["streams, status,<br/>state, or run directory"]
    verification["diagnostics, integrity,<br/>replay, or gate"]
    decision["bounded operational decision"]

    request --> contract --> execution --> outcome --> verification --> decision
    verification -->|"missing or inconsistent evidence"| refuse["refuse or narrow the claim"]
```

| Outcome | Trust it for | Do not infer |
| --- | --- | --- |
| successful `bijux` result | that one route completed under the observed state | that delegated code was isolated or all plugins are healthy |
| accepted DAG validation | that graph syntax and semantics passed the active contract | that any node executed |
| finalized run directory | that execution reached a retained terminal result | that files remain intact or domain output is correct |
| strict verification | that retained structural and integrity contracts pass | scientific, business, or workload correctness |
| replay or semantic comparison | the recorded identity and difference classification | equivalence outside the selected evidence and environment |
| green maintainer gate | the exact selected suite at the recorded revision | omitted platforms, external services, or broader release claims |

The [v0.4.0 Release Notes](bijux-dag/operations/v0-4-0-release-notes.md) define
the current DAG release. [Future Direction](bijux-dag/foundation/future-direction.md)
is non-binding direction; if it conflicts with the release boundary, the
narrower shipped claim wins.
