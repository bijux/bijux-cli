---
title: Documentation Deployment Workflow
audience: maintainers
type: reference
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-06
---

# Documentation Deployment Workflow

`.github/workflows/deploy-docs.yml` builds the handbook, uploads a GitHub Pages
artifact, and deploys that artifact through the Pages API. It is a managed
consumer of the shared `bijux-std` workflow; change the shared source and
refresh standards rather than editing this repository copy directly.

## Invocation And Permissions

- `workflow_dispatch` supports deliberate deployment from `main`, `master`, or
  a `v*` tag
- `workflow_call` allows an owning workflow to invoke the deployment
- `contents: read`, `pages: write`, and `id-token: write` are the complete
  workflow permissions
- concurrency is scoped by Git ref and cancels an older in-progress deployment

A manual dispatch from another branch fails explicitly instead of producing a
build that cannot deploy.

## Repository Configuration

`.github/docs-deploy.env` supplies the repository-specific commands and output:

| Setting | Current value | Purpose |
| --- | --- | --- |
| `BIJUX_DOCS_INSTALL_COMMAND` | `make gh-docs-install` | install and report the docs toolchain |
| `BIJUX_DOCS_BUILD_COMMAND` | `make docs-artifact-pages docs-artifact-pages-check docs-check` | generate artifact pages and run the strict documentation gate |
| `BIJUX_DOCS_SITE_DIR` | `artifacts/docs/site` | identify the Pages bundle |
| `BIJUX_DOCS_RUST_TOOLCHAIN` | `1.86.0` | match `rust-toolchain.toml` |

Repository or organization variables may supply configuration, but a non-empty
repository env value takes precedence. The workflow discovers fallback Make
targets only when no explicit command is configured.

## Build And Deployment Contract

1. Resolve command, toolchain, setup, site URL, and output-directory settings.
2. Install only the Python, uv, Node, and Rust toolchains required by the
   repository.
3. Run the configured install and build commands.
4. Resolve a directory that contains `index.html`.
5. Run the optional verification command when configured.
6. Upload that directory with `actions/upload-pages-artifact`.
7. Deploy it with `actions/deploy-pages`.

The workflow does not use `mkdocs gh-deploy`, create a documentation commit, or
configure a Git author. The uploaded site artifact is the deployment boundary.

## Diagnose A Failure

| Failure | First evidence to inspect |
| --- | --- |
| command discovery | resolved `install_command`, `build_command`, and `verify_command` outputs |
| toolchain setup | `.github/docs-deploy.env` and repository variables |
| strict build | the configured build command, normally `make docs-check` |
| artifact resolution | `BIJUX_DOCS_SITE_DIR` and whether it contains `index.html` |
| Pages upload | uploaded artifact path and `pages: write` permission |
| deployment | GitHub Pages environment status and OIDC permission |

## Related Operations

- [Documentation Operations](../operations/docs-operations.md)
- [CI Workflow](ci.md)
- [GitHub Release Workflow](release-github.md)
