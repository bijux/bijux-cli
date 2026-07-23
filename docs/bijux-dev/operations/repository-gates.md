---
title: Repository Gates
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-23
---

# Repository Gates

Repository gates matter because `bijux-core` does not treat CI as a ceremonial
green badge. The same evidence should support local review, continuous
integration, and release readiness.

## Choose The Smallest Honest Lane

| Gate | Scope | Cost and use |
| --- | --- | --- |
| `make fmt` | Rust formatting | fast pre-commit syntax and style proof |
| `make lint` | configured Rust lint policy | routine review gate after implementation changes |
| `make test` | required Rust release lane plus Python tests | normal repository test gate; governed slow Rust tests are excluded |
| `make test-slow` | governed slow Rust roster | deliberate expensive behavior, performance, and stress coverage |
| `make test-all` | complete Rust suite including ignored tests | broad local proof; not the default edit loop |
| `PINNED_REF=<commit> make test-all-frozen` | `test-all` from an immutable checkout | long-running evidence tied to an exact commit |
| `make docs-check` | source contracts, strict MkDocs build, publication budget, and navigation | every reader-facing documentation or navigation change |

`make test-all` is a complete Rust lane, not a claim that Python, docs, release,
or every governance command also ran. Select those gates separately when the
change crosses their ownership boundary.

```mermaid
flowchart TB
    change["Changed surface"]
    focused["Owning focused check"]
    rust["Rust lane<br/>fmt · lint · test · test-slow · test-all"]
    python["Python lane<br/>test-py · test-nightly-py"]
    docs["Documentation lane<br/>docs-check"]
    release["Release and governance verification"]
    report["Record exact commands, revision, and omissions"]

    change --> focused
    focused --> rust
    focused --> python
    focused --> docs
    focused --> release
    rust --> report
    python --> report
    docs --> report
    release --> report
```

Follow only the branches owned by the change, but follow every applicable
branch. A broad Rust run cannot compensate for omitted Python, documentation,
or release validation.

## Focused Tests First

For one failed Rust test, rerun its owning binary and test name before using a
root lane:

```bash
cargo nextest run -p bijux-dev \
  --test docs_source_reference_contracts \
  -E 'test(markdown_links_and_anchors_resolve)'
```

Focused success proves the repaired behavior. It does not replace broader
contract or repository validation when the underlying change affects those
surfaces.

## How To Read A Failure

When a gate fails, classify it before retrying:

1. formatting or static policy
2. documentation source, link, build, or navigation
3. contract, schema, generated reference, or retained evidence
4. runtime behavior
5. automation, environment, or toolchain

Classification controls which maintainer commands and handbook pages to use
next.

For frozen suites, use the printed console log, status file, and primary
nextest report under `artifacts/<commit>/`. Preserve the final nextest summary:
passed, failed, slow, skipped, and leaky counts are part of the evidence even
when the command exits unsuccessfully.

```mermaid
sequenceDiagram
    participant M as Maintainer
    participant L as Pinned gate launcher
    participant C as Immutable checkout
    participant N as nextest
    participant A as artifacts/commit

    M->>L: PINNED_REF=commit make test-all-frozen
    L->>C: verify or create clean detached checkout
    L-->>M: print PID, console, status, and artifact paths
    C->>N: make test-all
    N->>N: run complete selected suite
    N->>A: write full log and final summary
    C->>A: write terminal exit status
```

The launch command returning successfully proves only that the background
process started. Completion is established by the status file together with
the terminal nextest summary. Individual test failures do not suppress the
remaining selected tests or the summary.

## Evidence Semantics

| Signal | Accepted meaning | Excluded claim |
| --- | --- | --- |
| local gate success | selected checks passed on the recorded revision, worktree, toolchain, and host | hosted permissions, credentials, or runner behavior also work |
| CI gate success | selected checks passed under the recorded workflow, runner, permissions, and source identity | behavior outside the workflow matrix is covered |
| docs gate success | source contracts, links, strict build, publication boundary, and rendered navigation passed | every behavioral claim is true without owning product evidence |
| contract gate success | the selected public promises align with their schemas, fixtures, or implementations | unrelated runtime, security, or release surfaces are ready |
| release gate success | the candidate satisfied the declared pre-publication evidence set | external registries have accepted and reconciled every artifact |

## Select Gates By Risk

| Changed risk | Required starting evidence | Broader evidence when the boundary crosses |
| --- | --- | --- |
| public documentation | `make docs-check` | owning product contract for changed behavior claims |
| Rust runtime behavior | focused test, `make fmt`, and `make lint` | `make test`, slow roster, or complete Rust lane according to reach |
| Python distribution or bridge | focused pytest plus Python formatting and lint | `make test` when native parity or packaging is affected |
| graph, cache, replay, or artifacts | focused owning and join contracts | DAG contract/evidence family and required Rust lane |
| dependency or vulnerability | affected advisory and policy check | `make security` plus package or release verification |
| generated evidence or schema | named producer plus freshness contract | consumers, compatibility, and release evidence set |
| package or release surface | package plan and focused publication checks | complete release validation and external reconciliation |
| shared managed standard | upstream `bijux-std` proof and exact source SHA | downstream checksum and repository contract checks |

Gate selection follows the failure authority. A broad test run does not replace
the documentation, security, compatibility, or publication control that can
actually detect the risk.

## Reporting Rule

Report the exact command, commit, result, and any intentionally omitted lane.
Do not describe a focused test as a full gate, a complete Rust lane as a
repository-wide test, or a background launch as a successful result.

## Code Anchors

- `makes/rust.mk`
- `makes/dag.mk`
- `makes/docs.mk`
- `crates/bijux-dev/src/suites/`

## Authorities

- [Evidence Collection](evidence-collection.md)
- [Test Policy](../governance/test-policy.md)
- [Core Testing and Validation](../../bijux-core/operations/testing-and-validation.md)
