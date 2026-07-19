---
title: Documentation Operations
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-09
---

# Documentation Operations

Use this runbook when a documentation change must remain structurally valid,
locally reviewable, and publishable. The handbook is a product surface:
incorrect command guidance, unreachable pages, and stale release claims are
behavioral defects even when Markdown renders successfully.

## Choose The Local Command

| Need | Command | Result |
| --- | --- | --- |
| preview edited pages | `make docs-serve` | synchronized site with live reload on the first available local port |
| run the required gate | `make docs-check` | strict build, governed shell checks, badges, hygiene, publication boundary, and navigation checks |
| inspect maintainer governance | `cargo run -q -p bijux-dev --bin bijux-dev-cli -- docs-audit` | repository-specific documentation audit |
| produce the deployable site | `make docs` | clean build under `artifacts/docs/site` |

`make docs-check` installs the pinned documentation requirements before
building. Use the narrower publication or source-reference checks while
editing, but do not substitute them for the complete gate before handoff.

## Change Discipline

- update guidance in the same change as reader-visible behavior
- preserve one authority for each command, contract, or operational policy
- route generated outputs to `artifacts/` unless the repository governs a
  checked-in destination
- update `mkdocs.yml` when a public page is added, moved, or retired
- verify links after deleting or consolidating pages
- keep `.github/docs-deploy.env` aligned with the workspace toolchain and Make
  targets

## Review The Built Site

After `make docs-check`, inspect the result under `artifacts/docs/site`:

| Surface | Why it matters |
| --- | --- |
| entry pages | a new reader can identify the product, support level, and first action |
| navigation | canonical pages are reachable once and grouped by reader intent |
| commands | examples match current flags, paths, and output contracts |
| links | moved pages and anchors resolve without redirects or dead ends |
| claims | maturity, isolation, compatibility, and release language match implemented evidence |
| generated pages | published contracts and artifact summaries match their governed sources |

The full gate also checks that build output does not leak into root `site/`,
root `.cache/`, or a generated artifact directory beneath documentation
sources.

## Publication Boundary

Local validation ends at a complete site artifact. GitHub deployment is owned
by the managed [Documentation Deployment Workflow](../gh-workflows/deploy-docs.md),
which uploads `artifacts/docs/site` through GitHub Pages. It does not run
`make docs-deploy` or commit generated HTML.

## Code Anchors

- `mkdocs.yml`
- `mkdocs.shared.yml`
- `makes/docs.mk`
- `docs/automation/publish_contract_assets.py`
- `.github/docs-deploy.env`
- `.github/workflows/deploy-docs.yml`

## Related Standards

- [makes](../makes/index.md)
- [Documentation Standard](../governance/documentation-standard.md)
- [Core Documentation Standards](../../bijux-core/governance/documentation-standards.md)
- [CI and Automation](ci-and-automation.md)
