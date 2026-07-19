# Governance And Releases

Governance checks repository policy against current source. Release validation
assembles product, package, documentation, evidence, and automation facts into
an honest readiness decision.

## Governance Domains

The control plane enforces:

- package and dependency boundaries;
- source ownership and layout;
- command and public API surfaces;
- documentation references, page budgets, and generated authorities;
- evidence registries and consumer mappings;
- test lane, ignored-test, and slow-test discipline;
- Make and frozen-run status behavior;
- package metadata, version, publication, and release assets;
- synchronized standards integrity.

A governance failure is repaired at its owning source. Tests are not weakened
to accept drift.

## Shared Standards

Files synchronized from `bijux-std` are managed inputs. A defect in shared
policy is fixed upstream and refreshed from an accepted exact reference.
Downstream code may validate checksums and consumption but must not hand-edit
managed content.

Repository-specific policy remains local when it genuinely describes this
repository rather than an organization-wide standard.

## Release Inputs

Release readiness considers:

- clean and identified source state;
- workspace/package versions and toolchains;
- public versus private package boundaries;
- Rust and Python package metadata;
- command references and compatibility fixtures;
- required test, lint, docs, and security lanes;
- generated reports and evidence integrity;
- release notes, assets, provenance, and publication order;
- unresolved gaps and intentional differences.

One focused test or successful build cannot satisfy this aggregate.

## Status Semantics

Release status distinguishes ready, blocked, incomplete, advisory, stale, and
unknown evidence where the schemas define them. Missing required evidence is
not success. A failed component remains visible in summary and final nonzero
status.

Simulated, internal, narrowed, and skipped work is labeled and cannot support a
full public readiness claim.

## Mutation And Publication

Generators may prepare release trees, manifests, notes, provenance, and
reports under explicit commands. Publication remains an external authorized
operation; validation does not push, publish, tag, or mutate remote state.

Release-tree preparation uses committed source identity rather than silently
including a dirty worktree.

## Failure Ownership

- Product semantic failure: product crate.
- Packaging or command distribution failure: owning package.
- Report/generator defect: `bijux-dev`.
- Make or workflow status loss: wrapper owner.
- Shared policy defect: `bijux-std`.
- Missing evidence: producer and registry owner.

Do not add a local exception unless explicitly authorized and documented with
its durable upstream resolution.

## Verification

Release-validation suite, foundation hard-release, package-boundary, runtime
identity, version compatibility, test-lane, Make evidence, and synchronized
standards contracts protect this surface.
