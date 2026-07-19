---
title: Architecture Review Checklist
audience: maintainer
type: spec
status: canonical
owner: bijux-core-architecture
last_reviewed: 2026-07-19
---

# Architecture Review Checklist

Architecture review decides whether ownership, dependency direction, public
boundaries, state, and proof remain coherent. It is not satisfied by the
presence of architecture documents.

## Scope Declaration

Before review, record:

- affected product family and packages;
- public, private, experimental, simulated, or internal release lane;
- changed command, API, schema, state, artifact, backend, or build surface;
- source revision and relevant feature/platform selection;
- compatibility and migration expectations.

An undefined scope makes a green review meaningless.

## Ownership And Dependencies

- Every changed fact has one authority and one package owner.
- Dependency edges point toward semantic owners and obey public/private
  boundaries.
- Product crates do not acquire maintainer or testkit dependencies.
- Executable wrappers remain wiring; application or domain policy stays in
  libraries.
- Shared helpers do not erase domain ownership or create a catch-all package.
- Cross-package changes update each affected contract without duplicating
  authority.

## State, Effects, And Failure

- Persistent state has schema, identity, lifecycle, and migration ownership.
- Filesystem, process, network, clock, and environment effects remain behind
  explicit boundaries.
- Concurrent operations own isolated resources or implement reviewed
  coordination.
- Failure classes preserve whether admission, execution, persistence, or
  presentation failed.
- Recovery does not rewrite historical evidence silently.
- Security and backend capability claims state what is enforced and what
  remains ambient.

## Compatibility And Release

- Machine-readable package and release contracts agree with manifests.
- Public commands, output fields, schemas, and artifacts have compatibility
  treatment.
- Internal or modeled surfaces do not appear in stable help or documentation.
- Release automation covers every public package in governed order.
- Migration or refusal behavior exists for incompatible persisted state.
- Known limitations and residual risks remain visible.

## Executable Evidence

| Change class | Required proof |
| --- | --- |
| dependency or package boundary | metadata and forbidden-dependency contracts |
| command or API | focused contract, integration, and generated-reference checks |
| schema or artifact | round-trip, compatibility, corruption, and strict verification |
| scheduler or backend | state transition, cancellation, retry, and backend evidence |
| documentation authority | source-link, contract, publication, and strict build checks |
| release surface | isolated package, artifact, and dry-run publication evidence |

Reviewers inspect assertions and failure behavior, not only test names or
counts.

## Evidence Integrity

- Commands, terminal statuses, source revision, and retained paths are recorded.
- Failed, skipped, ignored, slow, leaky, unavailable, and advisory outcomes
  remain visible.
- Generated reports identify a real producer and are refreshed from it.
- Curated assessments identify their source authorities and review revision.
- Expected fixtures are changed only after semantic review.

## Exit Decision

Architecture review passes only when ownership, dependencies, compatibility,
effects, failure behavior, executable evidence, and documentation agree for the
declared scope. It fails when a required authority is absent, a dependency
violates direction, evidence contradicts the claim, or a limitation is hidden.
An explicit “incomplete” decision is correct when required evidence was not
run; it must not be relabeled as pass.
