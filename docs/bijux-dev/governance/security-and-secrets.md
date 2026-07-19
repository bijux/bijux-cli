---
title: Security and Secrets
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Security And Secrets

Maintainer automation can publish packages, write releases, upload containers,
and retain evidence. Credentials for those operations are capabilities, not
configuration convenience. Their scope, lifetime, and exposure path must match
one owned operation.

## Credential Boundaries

| Capability | Preferred authority | Must not appear in |
| --- | --- | --- |
| GitHub release and repository writes | job-scoped `GITHUB_TOKEN` permissions | source, local config examples, or uploaded logs |
| crates.io publication | release secret or approved trusted identity | Cargo config committed to the repository |
| PyPI publication | trusted publishing with environment protection | package metadata, wheel contents, or command arguments |
| container publication | job-scoped package permission | image layers, build arguments, or retained environment dumps |
| local maintainer access | environment or OS credential store | shell history, reports, fixtures, or copied diagnostics |

Prefer short-lived identity and trusted publishing over reusable bearer tokens.
When a token is unavoidable, grant only the registry, package, and operation
needed by the job.

## Workflow Review

For every workflow that can read a secret or write externally, verify:

1. The triggering event cannot execute untrusted pull-request code with the
   credential.
2. Permissions are declared at the workflow or job and are no broader than the
   operation requires.
3. The secret is not passed through command-line arguments, debug traces,
   caches, matrices, artifacts, or generated reports.
4. Fork and dependency behavior cannot replace the code that receives the
   credential.
5. Publication is tied to an accepted source revision and records artifact
   identity.
6. Failure and retry behavior cannot publish a conflicting or unreviewed
   artifact.

`pull_request_target` requires particular scrutiny because it runs with base
repository authority. Never combine that authority with checkout or execution
of untrusted head-revision code.

## Local And Generated Evidence

- Direct command output, caches, and reports to `artifacts/`.
- Treat environment dumps as sensitive until reviewed and redacted.
- Use synthetic credentials in tests and documentation.
- Do not retain complete request headers, registry configuration, or credential
  store paths in governed reports.
- Redaction is a display control, not proof that the underlying value was never
  stored or transmitted.

Before committing generated evidence, inspect both the content and its
producer. A report that hides one known token format can still expose another
secret, a private path, or a credential-bearing URL.

## Incident Handling

If exposure is suspected:

1. Stop the affected workflow or publication path without deleting evidence.
2. Revoke or rotate the credential at its authority.
3. Record affected repositories, packages, registries, runs, and time window.
4. Preserve workflow logs, artifact digests, release identities, and audit
   records with access limited to responders.
5. Determine whether unauthorized artifacts or releases were published.
6. Repair the exposure path and add a regression control before restoring
   publication.
7. Follow the root [`SECURITY.md`](../../../SECURITY.md) disclosure process
   when users or supported artifacts may be affected.

Rotation limits future use; it does not remove a published artifact, erase a
log, or prove the credential was not used.

## Review Anchors

- `.github/workflows/`
- `.github/release.env`
- `makes/gh.mk`
- [CI and Automation](../operations/ci-and-automation.md)
- [Incident Response](../operations/incident-response.md)
- [Release Operations](../operations/release-operations.md)
