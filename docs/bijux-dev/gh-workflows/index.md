---
title: GitHub Workflow Ownership
audience: maintainers
type: index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# GitHub Workflow Ownership

Start with the workflow family and its authority before debugging an individual
job. Some workflows are repository-owned; others are synchronized from
`bijux-std` and must be corrected upstream rather than patched in this checkout.

## Validation And Policy

| Workflow | Responsibility | Detailed guide |
| --- | --- | --- |
| `ci.yml` | required formatting, lint, security, and test jobs | [CI](ci.md) |
| `release-validation.yml` | committed-source release-candidate proof | [Release Validation](release-validation.md) |
| `bijux-canon.yml` | broader canon and evidence integration | [Bijux Canon](bijux-canon.md) |
| `bijux-std-checks.yml`, `bijux-std.yml` | shared standards and contract validation | [CI and Automation](../operations/ci-and-automation.md) |
| `github-policy.yml`, `pr-approval-policy.yml`, `automerge-pr.yml` | repository settings, review policy, and approved merge automation | [CI and Automation](../operations/ci-and-automation.md) |

Policy and standards workflows are managed surfaces. Confirm the generated-file
notice and checksum before deciding where a change belongs.

## Documentation And Publication

| Workflow | Responsibility | Detailed guide |
| --- | --- | --- |
| `deploy-docs.yml` | strict build, Pages artifact, and deployment | [Documentation Deployment](deploy-docs.md) |
| `release-on-tag.yml` | release fan-out for an accepted tag | [Release Surfaces](../makes/release-surfaces.md) |
| `release-artifacts.yml` | reusable package artifact build | [Release Operations](../operations/release-operations.md) |
| `release-crates.yml` | crates.io publication | [Rust Crates Release](release-crates.md) |
| `release-pypi.yml` | Python package publication | [PyPI Release](release-pypi.md) |
| `release-ghcr.yml` | container publication | [Release Operations](../operations/release-operations.md) |
| `release-github.yml` | GitHub release and attached artifacts | [GitHub Release](release-github.md) |

## Failure Routing

1. Record the workflow, job, source commit, event, and final status.
2. Identify the delegated make target or called workflow.
3. Reproduce the repository-owned target locally when credentials and hosted
   policy are not the failing boundary.
4. If the file is managed by shared standards, verify drift and prepare the fix
   in `bijux-std`.
5. If publication began, inspect every target registry before retrying.

A failure spanning several workflows is usually a shared gate, release identity,
or standards problem. Do not copy a local workaround into each workflow.
