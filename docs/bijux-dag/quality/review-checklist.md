---
title: Review Checklist
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Review Checklist

This page is for independent review. Do not repeat the author's command list
and call that review; verify that the selected evidence can support the stated
behavior and compatibility claim.

## Ownership

- The change belongs to the crate and module that own the behavior.
- Higher layers orchestrate lower layers rather than reimplementing their
  semantics.
- Product crates do not acquire testkit or maintainer dependencies.
- A new file, module, contract, or report has one durable responsibility and
  a named consumer.

## Contract And Compatibility

- Input acceptance, identity, retained evidence, response shape, error code,
  exit behavior, and public imports were considered where relevant.
- Old fixtures or bundles are tested when compatibility is claimed.
- Unsafe old evidence is refused explicitly rather than silently
  reinterpreted.
- Experimental, simulated, or internal behavior is not described as stable.
- Breaking or release-facing changes update the owning changelog and release
  boundary.

## Evidence Authenticity

- Focused tests exercise the changed boundary rather than only a helper that
  validates hard-coded success fields.
- Snapshots retain semantic differences and do not normalize the assertion
  away.
- Generated files identify their source and producer, and the generated diff
  was reviewed separately from handwritten changes.
- A started background process, created report, or zero exit without final
  integrity checks is not reported as completed proof.
- Broad lanes preserve individual failures and final status.

## Operations And Security

- Filesystem, environment, clock, process, network, and container effects cross
  an explicit owned boundary.
- New subprocess or adapter behavior records cancellation, timeout, exit, and
  retained failure semantics.
- User-controlled paths and output remain within the documented artifact and
  run roots.
- Isolation claims match the implementation; local shell execution is not
  presented as sandboxed.

## Documentation Structure

The durable DAG handbook sections are `foundation`, `architecture`,
`interfaces`, `operations`, `quality`, and `packages`. Section page counts are
not fixed. Add a page only when it has a distinct reader question that cannot
be answered clearly by an existing owner.

Review that:

- every handbook page is a direct child of its owning section directory;
- public navigation remains below the repository page budget;
- internal specifications and generated reports remain outside public MkDocs
  unless deliberately curated;
- links, examples, source anchors, frontmatter, and release claims are valid;
- crate-local docs remain below ten substantive pages per crate.

## Decision

Request changes when evidence cannot support the claim, even if every selected
command is green. Approve only when ownership, behavior, compatibility,
documentation, and retained verification agree.

Record any unrun lane or unresolved risk explicitly. Silence is not acceptance.

## Related Guidance

- [Change Validation](change-validation.md)
- [Definition Of Done](definition-of-done.md)
- [Documentation Standards](documentation-standards.md)
- [Dependency Governance](dependency-governance.md)
