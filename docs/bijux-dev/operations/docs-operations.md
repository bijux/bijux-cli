---
title: Documentation Operations
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Documentation Operations

Use this runbook to prove that a documentation change is authoritative,
reachable, structurally governed, and publishable. The final review still
requires reading the changed pages and built site; automation cannot determine
whether prose overstates support or gives a useful recovery path.

## Validation Lanes

| Intent | Command | What it proves |
| --- | --- | --- |
| run handbook metadata, authority, orphan, depth, and page-budget checks | `make docs-governance-lint` | repository documentation policy |
| regenerate governed inventory and consolidation evidence | `make docs-inventory-generate` | current inventory reports from the owning producer |
| run the required documentation gate | `make docs-check` | synchronized shell, badges, strict MkDocs build, output hygiene, publication boundary, and rendered navigation |
| build the site for inspection | `make docs` | site output under `artifacts/docs/site` |
| inspect changes with live reload | `make docs-serve` | local preview; not a validation result |

Run the narrow governance lane while editing. Run `make docs-check` before
handoff. Regenerate inventories only when the source set or governed inventory
format changes; do not create report churn for prose edits that leave inventory
unchanged.

## Choose The Owning Surface

| Material | Destination |
| --- | --- |
| reader-facing repository, CLI, DAG, or maintainer guidance | corresponding `docs/bijux-*` handbook |
| executable prose contract consumed by tests or tools | `docs/spec` |
| reproducible evidence compared across revisions | `docs/reports` |
| package purpose, public imports, and package-local verification | crate README or crate-local docs |
| local logs, screenshots, built sites, and one-off analysis | `artifacts/` |
| future product direction | owned roadmap with explicit non-binding status |

Do not move `docs/spec` or `docs/reports` into a handbook to make the tree look
uniform. Confirm path consumers before moving either surface.

## Review The Source

Before building:

1. Search for duplicate authorities and stale path references.
2. Confirm every changed behavior claim against code, schema, command output,
   or an executable contract.
3. Check stable, experimental, simulated, internal, and unsupported wording
   against the current release boundary.
4. Verify commands from the repository root with generated output under
   `artifacts/`.
5. Confirm new pages meet admission criteria and have a deliberate inbound
   navigation route.
6. Confirm removed pages have no source, test, generator, workflow, or external
   contract consumer.

## Review The Built Site

Inspect `artifacts/docs/site` after `make docs-check`:

- Home and handbook entry pages identify the product and first useful action.
- Navigation exposes canonical reader decisions without publishing internal
  specs or reports.
- Tables and diagrams remain readable at desktop and mobile widths.
- Commands, links, anchors, generated references, and contract assets resolve.
- Security, compatibility, isolation, and maturity claims match current
  evidence.
- Removed and consolidated pages do not leave dead navigation or duplicate
  search results.

## Generated And Managed Content

`docs/automation/publish_contract_assets.py` publishes governed contract assets
for the site. Inventory commands own their checked-in report paths. Shared
theme and workflow content comes from `bijux-std`.

Change a producer before its generated output. Review the semantic diff, run
the owning contract, and keep independently meaningful producer and generated
changes in separate commits. Do not hand-edit `.bijux/shared/` or generated
GitHub standards.

## Handoff Evidence

Report:

- exact documentation commands and outcomes;
- public page count and maximum product-tree depth;
- maximum crate-local docs count;
- whether generated inventories changed;
- any skipped manual or automated check;
- unresolved source/documentation contradiction.

The deployment workflow publishes the built Pages artifact. Local
`make docs-deploy` is not part of the ordinary pull-request proof.

## Authorities

- [Documentation Standards](../../bijux-core/governance/documentation-standards.md)
- [Maintainer Documentation Standard](../governance/documentation-standard.md)
- [Documentation System](../../bijux-core/foundation/documentation-system.md)
- [CI and Automation](ci-and-automation.md)
- [Documentation Deployment](../gh-workflows/deploy-docs.md)
