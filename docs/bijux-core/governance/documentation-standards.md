---
title: Documentation Standards
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# Documentation Standards

Documentation is part of the product contract. A page earns its place by
answering a distinct reader question with current behavior, a clear owner, and
a way to detect drift. Formatting, frontmatter, and a passing build are
necessary but do not make weak content acceptable.

## Documentation Surfaces

| Surface | Reader and authority | Publication |
| --- | --- | --- |
| root `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, and release history | repository entry, contribution, disclosure, and release record | GitHub repository |
| `docs/bijux-core`, `docs/bijux-cli`, `docs/bijux-dag`, `docs/bijux-dev` | curated reader handbooks | selected explicitly by `mkdocs.yml` |
| `docs/spec` | executable prose contracts consumed by tests and tooling | excluded from the public site |
| `docs/reports` | revision-bound generated or governed evidence | excluded from the public site |
| crate `README.md`, `CHANGELOG.md`, and crate-local `docs/` | package consumers and maintainers | crates.io, docs.rs, PyPI, or repository source as applicable |
| `artifacts/` | local output, logs, built sites, and transient analysis | never a tracked documentation authority |

Do not merge specifications or reports into a handbook merely to simplify the
filesystem. Their authority and lifecycle differ. Do not copy their normative
text into a public page; explain supported behavior and link to the repository
source when deeper review is necessary.

## Page Admission

Add a durable page only when all of these are true:

1. A named audience has a question not already owned by another page.
2. The answer is stable enough to maintain across releases.
3. Code, schema, command, test, or governance evidence can expose drift.
4. The page has a durable owner and an intended navigation route.
5. Adding it is better than extending or replacing an existing authority.

A new heading, initiative, package, or report does not automatically require a
new page. If the content cannot survive without phrases such as "use this page
when" and a generic list of anchors, the reader question is probably not yet
clear enough.

## Publication Budget

The governed policy in `configs/dag/policy/docs_lint_policy.json` limits:

- the public MkDocs navigation to 100 Markdown pages;
- product handbooks to the product/category/page shape illustrated by
  `docs/bijux-core/architecture/system-overview.md`;
- each crate-local `docs/` tree to ten Markdown pages.

The budget is a ceiling, not a target. A public addition should displace,
consolidate, or justify itself against existing navigation. Internal
specifications and generated reports do not consume the public-page budget, but
they still require authority, provenance, and retention discipline.

The four governed handbook roots are:

- `docs/bijux-core/`
- `docs/bijux-cli/`
- `docs/bijux-dag/`
- `docs/bijux-dev/`

Each category contains pages directly. Do not add another durable directory
layer beneath a category to encode initiative, delivery order, or temporary
ownership.

## Content Acceptance

A canonical page must:

- state current behavior before background or aspiration;
- distinguish stable, experimental, simulated, internal, and unsupported
  surfaces;
- name the component or process that owns the behavior;
- explain failure, refusal, recovery, or release consequence where relevant;
- use executable examples when the reader is expected to run a workflow;
- cite real repository paths only when they help verify or change the behavior;
- avoid decorative diagrams, invented measurements, placeholder sections, and
  repeated navigation prose.

Frontmatter under the four handbook roots must contain `title`, `audience`,
`type`, `status`, `owner`, and `last_reviewed`. A current review date means the
reviewer checked behavior and references; changing the date alone is not a
review.

## Authority Conflicts

Machine schemas own serialized shape. Executable specifications and tests own
enforced behavior. Handbooks own supported reader-facing explanations. Package
pages own package-local use and boundaries. Reports retain evidence for a
revision.

When two surfaces disagree, determine the intended behavior and repair every
affected authority and consumer. Do not preserve contradictory text because
one copy is generated, or weaken a test to make stale prose pass.

## Consolidation And Removal

Merge or remove a page when it:

- duplicates another authority;
- has no distinct reader decision;
- consists mainly of headings, generic bullets, or path catalogs;
- describes a capability outside the release boundary without a necessary
  limitation or roadmap role;
- has no inbound route and no executable consumer;
- records transient status better kept in an issue or artifact.

Before removal, search Markdown, source, tests, generators, and workflow files
for path consumers. Stable `docs/spec` and `docs/reports` paths can be
interfaces; moving them requires updating producers and contract tests in the
same change.

## Required Review

Run `make docs-check` before handoff. Inspect the built entry pages and changed
navigation under `artifacts/docs/site`; a strict build cannot judge whether a
claim is honest or an example is useful.

Use the [Documentation System](../foundation/documentation-system.md) for the
authority order and [Documentation Operations](../../bijux-dev/operations/docs-operations.md)
for commands and evidence.
