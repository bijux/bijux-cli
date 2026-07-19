---
title: Architecture Risks
audience: mixed
type: architecture
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# Architecture Risks

This register covers repository-level failure modes that can invalidate more
than one package or release surface. Product-specific operational risks remain
in the CLI and DAG risk registers.

## Active Risks

| Risk | Failure mode | Detection | Release consequence |
| --- | --- | --- | --- |
| package-boundary drift | a public crate gains a runtime or build dependency on a private crate, or publish order becomes invalid | `foundation_workspace_package_boundary_contracts` compares the governed boundary with Cargo metadata | block crate publication until ownership and dependency direction agree |
| product/maintainer coupling | user runtime code imports repository proof logic or maintainer commands become hidden product entrypoints | source-layout guardrails and package-boundary tests | block the affected runtime release |
| contract divergence | code, schema, generated reference, and handbook describe different behavior | owning contract tests, generated-reference checks, and `make docs-check` | block publication of every surface carrying the inconsistent behavior |
| release identity split | crates, Python package, containers, docs, or evidence are produced from different source identities | release plans, tag checks, package metadata checks, and retained release evidence | discard or rebuild artifacts; never publish a mixed release set |
| retained-evidence ambiguity | reports or artifacts omit provenance, integrity state, or the contract used to interpret them | report producers, artifact integrity checks, and evidence governance contracts | evidence cannot support a release decision |
| configuration drift | local, CI, and release invocations resolve different effective inputs without making the difference visible | configuration precedence contracts and reproducible gate commands | investigate and align inputs before accepting a green result |

## Acceptance Standard

A risk is not mitigated because a handbook says that a control exists.
Mitigation requires all of the following:

- one owner can change the behavior and the control;
- an executable check fails when the invariant is violated;
- the failure names the affected surface rather than reporting a generic gate
  error;
- release guidance states whether the failure blocks publication;
- any accepted exception has a bounded scope, evidence, and removal condition.

## Review Triggers

Review this register when a change:

- adds a workspace crate or changes publication metadata;
- introduces a dependency across product or maintainer families;
- changes a public schema, output envelope, or generated reference;
- adds a release channel or changes artifact provenance;
- creates a new checked-in report or evidence class;
- changes configuration precedence used by CI or release automation.

## Evidence Boundaries

The risk register identifies what must be detected; it is not proof that the
controls passed. Use the current test output and generated evidence for the
reviewed commit. A report from another revision, a focused test presented as a
full gate, or a successful background launch without its final status is not
release evidence.

## Related Authorities

- [Dependency Direction](dependency-direction.md)
- [Artifact and Contract Flow](artifact-and-contract-flow.md)
- [Risk and Exceptions](../governance/risk-and-exceptions.md)
- [Testing and Validation](../operations/testing-and-validation.md)
- [Documentation System](../foundation/documentation-system.md)
