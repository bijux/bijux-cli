---
title: Documentation Root Inventory Report
audience: maintainer
type: report
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# Documentation Root Inventory Report

## Purpose

This report records the permitted documentation roots and their authority. It
prevents tutorials, executable specifications, generated evidence, internal
crate notes, and local output from collapsing into one public tree.

## Root Classification

| Root | Audience | Authority | Publication |
| --- | --- | --- | --- |
| `docs/index.md` | all readers | site landing and handbook routing | public |
| `docs/bijux-core/` | contributors and maintainers | repository scope, architecture, governance, and operations | curated public |
| `docs/bijux-cli/` | CLI users and integrators | `bijux` product guidance | curated public |
| `docs/bijux-dag/` | DAG users and integrators | `bijux-dag` product guidance | curated public |
| `docs/bijux-dev/` | repository maintainers | maintainer command, gate, workflow, and evidence guidance | curated public |
| `docs/spec/` | maintainers and executable checks | normative cross-package specifications | excluded from public navigation |
| `docs/reports/` | maintainers and review automation | governed observations, inventories, ledgers, and assessments | excluded from public navigation |
| `docs/assets/` | documentation runtime | committed images, styles, and browser assets | support files |
| `docs/automation/` | documentation maintainers | site validation and generation scripts | not reader content |
| `docs/overrides/` | documentation runtime | MkDocs template overrides | support files |
| crate-local documentation directories | crate developers | package-local architecture and contracts | internal, linked from crate README files |
| `artifacts/docs/` | local tooling | generated sites and check output | disposable, never authority |

## Public Shape Contract

Each of the four public handbook roots contains `index.md` plus durable
category directories. Published pages may use only product/category/page
depth. `mkdocs.yml` is the publication allowlist and excludes `/spec/` and
`/reports/`.

`make docs-publication-check` enforces:

- at least 40 and at most 100 public Markdown pages;
- no public handbook page deeper than product/category/page;
- required authority pages for product boundaries and limitations;
- explicit exclusion of internal specifications and reports;
- rejection of stock, formulaic presentation prose in the repository handbook.

## Placement Decisions

| Content | Destination |
| --- | --- |
| first-use or operational guidance | owning public handbook |
| package implementation boundary | owning crate `README.md` and `docs/` |
| normative cross-package state, schema, or compatibility rule | `docs/spec/` |
| generated or checked observation | `docs/reports/` with producer or validator |
| local build, log, report, or rendered site | `artifacts/` |
| shared Bijux shell or standard | upstream `bijux-std`, consumed under `.bijux/shared/` |

## Review Criteria

The inventory is healthy only when every root has one authority, public
navigation stays within budget, internal files remain reachable from an index
or executable consumer, crate-local docs stay within their page limit, and
generated output does not appear in a source root. Existence alone is not
quality: stale, duplicate, formulaic, or ownerless pages still fail review.
