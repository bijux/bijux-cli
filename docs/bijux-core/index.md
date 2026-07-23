---
title: Repository Handbook
audience: mixed
type: index
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-23
---

# Repository Handbook

`bijux-core` is the release and compatibility boundary for two public
command-line products. It keeps runtime code, machine-readable contracts,
retained evidence, and publication automation in one workspace so a shipped
claim can be traced to both an implementation owner and a verification owner.

<div class="bijux-callout"><strong>The repository boundary starts where one
product handbook stops.</strong> Cross-product contracts, package publication,
dependency direction, shared automation, and release evidence are governed
here without merging the two runtime authorities.</div>

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="foundation/platform-overview/">Explore the platform</a>
<a class="md-button" href="architecture/system-overview/">Trace ownership</a>
<a class="md-button" href="operations/testing-and-validation/">Select verification</a>
</div>

## Start Here

| Question | Best starting page |
| --- | --- |
| What does `bijux-core` publish today? | [Platform Overview](foundation/platform-overview.md) |
| Which package owns this behavior? | [Package Map](foundation/package-map.md) |
| Which crates are public and which stay internal? | [Package Boundary](foundation/package-boundary.md) |
| How is the workspace laid out and why? | [System Overview](architecture/system-overview.md) |
| What do contributors run before review or release? | [Testing and Validation](operations/testing-and-validation.md) |

## Repository Snapshot

`bijux-core` publishes two public product families:

- `bijux`, the operator-facing command runtime
- `bijux-dag`, the local-first DAG runtime and crate family

It also carries repository-internal support crates that make those products
shippable and auditable:

- `bijux-cli-python` for Python packaging and bridge parity
- `bijux-dag-testkit` for deterministic DAG fixtures and shared assertions
- `bijux-dev` for repository diagnostics, governance, evidence, and release
  tooling

That split matters because many questions that look like one command problem
are really package-boundary or release-boundary questions.

```mermaid
flowchart TB
    repo["bijux-core workspace"]

    subgraph public["Published product families"]
        cli["bijux<br/>bijux-cli"]
        dag["bijux-dag<br/>core · artifacts · runtime · app · cli"]
    end

    subgraph private["Repository-only support"]
        py["bijux-cli-python<br/>Python distribution build"]
        testkit["bijux-dag-testkit<br/>fixtures and assertions"]
        dev["bijux-dev<br/>governance and evidence"]
    end

    repo --> public
    repo --> private
    py --> cli
    testkit -. verifies .-> dag
    dev -. governs release evidence .-> public
```

This is an ownership map, not a Cargo dependency graph. Use the
[Package Map](foundation/package-map.md) for package purpose and the
[Package Boundary](foundation/package-boundary.md) before treating a private
support crate as a distributable API.

## Repository Control Loops

Four connected loops keep the workspace coherent:

| Loop | Input | Accepted output | Refusal condition |
| --- | --- | --- | --- |
| runtime | command or graph input | streams and exit status, or a retained DAG run | parsing, validation, execution, integrity, or policy failure |
| contract | schema, invariant, or compatibility rule | machine-checkable agreement between producer and consumer | drift, unknown versions, incompatible meaning, or missing ownership |
| evidence | source revision plus named scenario or suite | retained observation with producer, selection, and status | stale, partial, unverifiable, or non-comparable result |
| release | accepted package set and version | dependency-ordered registry and release artifacts | missing gate, version disagreement, partial publication, or unsupported claim |

The loops are intentionally separate. A runtime success does not satisfy a
release gate. A generated report does not redefine the contract it evaluates.
A private maintainer command cannot widen the product surface.

```mermaid
flowchart LR
    change["owned source change"]
    contract["contract and compatibility review"]
    focused["owner verification"]
    evidence["retained evidence"]
    release["release decision"]
    publish["publication"]

    change --> contract --> focused --> evidence --> release --> publish
    focused -->|"failure"| change
    evidence -->|"stale or incomplete"| focused
    release -->|"claim not proven"| change
```

## Boundary Promises

- Product crates never depend on `bijux-dev` for runtime behavior.
- Public and private package status is explicit and contract-tested.
- Generated references describe the live binaries instead of a manually
  maintained command inventory.
- DAG evidence is retained separately from source and local build products.
- A release claim names its supported lane and does not inherit experimental,
  simulated, or maintainer-only capabilities.
- Partial publication is handled as an incident, not retried as though no
  external state changed.

## Claim Ownership

| Claim | Implementation owner | Proof owner |
| --- | --- | --- |
| a `bijux` route behaves consistently | `bijux-cli` | CLI integration and contract suites |
| Python and Rust installations expose one CLI contract | `bijux-cli-python` and `bijux-cli` | bridge parity and packaging checks |
| graph meaning is deterministic | `bijux-dag-core` | graph, canonicalization, and planner contracts |
| an execution result is reusable | `bijux-dag-runtime` and `bijux-dag-artifacts` | cache, replay, lineage, and integrity contracts |
| a backend is supported | runtime backend plus application routing | conformance, real-substrate evidence, and release truth table |
| a release is publishable | package and release contracts | `bijux-dev`, make targets, and hosted release validation |

Implementation and proof are deliberately different responsibilities. A test
or generated report can reject a claim, but it cannot silently redefine the
runtime contract to make the claim pass.

## Product And Maintainer Handbooks

| Scope | Handbook |
| --- | --- |
| `bijux` routing, config, plugins, state, REPL, and output | [CLI Handbook](../bijux-cli/index.md) |
| graph authoring, execution, artifacts, cache, replay, and comparison | [DAG Handbook](../bijux-dag/index.md) |
| repository gates, evidence generation, automation, and release response | [Maintainer Handbook](../bijux-dev/index.md) |
