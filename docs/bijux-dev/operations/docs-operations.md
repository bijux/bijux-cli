---
title: Docs Operations
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-09
---

# Docs Operations

Use this page when documentation is changing and you need to protect handbook
structure, navigation, and publishability instead of just writing markdown that
looks acceptable in one editor tab.

Documentation operations matter because the handbook is part of the product
surface. Broken navigation, stale links, or misleading release guidance can be
just as damaging as broken code paths.

## Operational Rules

- handbook structures must match documented section contracts
- MkDocs navigation must include all canonical pages
- docs changes must ship with behavior changes in the same pull request
- `.github/docs-deploy.env` must keep `BIJUX_DOCS_RUST_TOOLCHAIN` aligned with the workspace Rust version

## Documentation Preflight

Before merging docs-heavy changes:

1. run `make docs-check`
2. confirm nav entries match filesystem paths
3. confirm no page links reference retired documents
4. confirm style and tone follow handbook standards

## What Reviewers Should Check

| Surface | Why it matters |
| --- | --- |
| navigation and filesystem alignment | readers must be able to find canonical pages consistently |
| docs and behavior coupling | public guidance should move with the feature it describes |
| toolchain alignment | docs deploys must use the same governed Rust baseline the repo documents |

## Standard Commands

```bash
make docs-check
make docs-serve
cargo run -q -p bijux-dev --bin bijux-dev-cli -- docs-audit
```

## Reader Shortcut

If a code change alters a reader-facing command, workflow, or release claim and
the docs move later, the documentation is already behind. The right time to fix
the handbook is inside the same change set.

## Code Anchors

- `mkdocs.yml`
- `mkdocs.shared.yml`
- `makes/docs.mk`
- `docs/automation/publish_contract_assets.py`

## Continue Reading

- [makes](../makes/index.md)
- [gh-workflows](../gh-workflows/deploy-docs.md)
- [Documentation Standard](../governance/documentation-standard.md)
- [Core Documentation Standards](../../bijux-core/governance/documentation-standards.md)
- [CI and Automation](ci-and-automation.md)
