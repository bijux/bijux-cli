---
title: Security and Secrets
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-09
---

# Security and Secrets

Use this page when a maintainer workflow touches credentials, release tokens,
or generated evidence and the real question is: what must stay private so the
repository can still trust its own automation?

Security and secrets governance exists to protect three things at once:
publication authority, evidence integrity, and operator confidence that green
automation still means something.

## Security Rules

- do not embed secrets in source or generated artifacts
- use scoped credentials with least-privilege access
- sanitize logs and reports to avoid accidental leakage
- rotate credentials after incident response events

## Where Exposure Usually Happens

- CI workflow secrets and release tokens
- local maintainer environments and shell history
- generated reports that may include sensitive paths or identifiers

## What Maintainers Should Check

| Surface | Why it matters |
| --- | --- |
| workflow changes | a harmless-looking automation edit can widen secret exposure |
| local helper commands | shell history, temp files, and copied output often leak before code does |
| generated evidence | reports must stay useful without disclosing credentials or internal-only identifiers |

## Reader Shortcut

If a workflow requires a broader secret scope than the publication or evidence
step it supports, the workflow is the problem. Do not normalize oversized
credentials just because the happy path currently works.

## Code Anchors

- `.github/workflows/`
- `crates/bijux-dev/src/tooling/git.rs`
- `crates/bijux-dev/src/tooling/cargo.rs`

## Continue Reading

- [Incident Response](../operations/incident-response.md)
- [Risk and Exceptions](../../bijux-core/governance/risk-and-exceptions.md)
- [Release Operations](../operations/release-operations.md)
