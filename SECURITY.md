# Security Policy

This repository uses coordinated vulnerability disclosure. Security reports are
handled privately until we understand impact and have a remediation path.

This repository is explicit about one important trust boundary:

- the runtime can execute installed plugins
- plugins are not sandboxed
- installing a plugin is a trust decision, not a security boundary

## What This Policy Covers

Security reports may cover:

- this monorepo and shared tooling under `makes/`, `configs/`, and `docs/`
- workspace crates under `crates/`, including `bijux-cli`, `bijux-cli-python`,
  and DAG runtime crates
- published artifacts produced from official tagged releases in this repository

## What To Report

Examples of in-scope reports include:

- authentication or authorization bypass
- unsafe defaults that expose data or execution surfaces
- supply-chain weaknesses in build, publish, or artifact handling
- secrets exposure in tracked files or generated release artifacts

## Supported Versions

Security fixes are applied to the latest released `bijux-cli` runtime only.
For this policy, "released" means an official tagged release with published
artifacts from this repository. Older versions may not receive patches.
Development branches, untagged local checkouts, and workspace-only maintainer
tooling are reviewed on a best-effort basis.

## Reporting a Vulnerability

Preferred:
- GitHub private report: https://github.com/bijux/bijux-core/security/advisories/new

Fallback:
- Email: [bijan@bijux.io](mailto:bijan@bijux.io)

Please include:
- affected version and install method
- whether the issue was observed on an official tagged release, a local checkout,
  or a workspace build
- operating system and runtime details
- clear reproduction steps
- expected impact
- PoC (if available)

Do not include secrets or private user data in reports.

## Response Expectations

This project is maintained on a best-effort basis.

Current targets:
- acknowledgement within 48 hours
- initial assessment within 5 business days

Complex issues can take longer to fix.

## Disclosure

Please do not disclose publicly before a fix or mitigation is available.
We will publish a GitHub security advisory when appropriate.

## Scope

In scope:
- this repository
- official release artifacts published from tagged releases in this repository
- the runtime, Python compatibility package, and repository-owned docs or build flows when they affect supported release artifacts on Linux or macOS

Out of scope:
- vulnerabilities in third-party plugins not maintained here
- the trust or behavior of locally installed untrusted plugins
- unsupported Windows runtime behavior
- untagged local checkout version strings or source-tree-only release preparation state by themselves
- social engineering and physical attacks
- denial-of-service load testing
- third-party infrastructure outside this project

## Notes

- No public bug bounty program.
- Non-security questions should go to regular GitHub issues.
- If the report depends on a third-party plugin, say whether the issue is in the
  runtime host or in the plugin itself.
- A host-side bypass of the documented plugin trust boundary is in scope; the
  ordinary risk of installing an untrusted plugin is not.
