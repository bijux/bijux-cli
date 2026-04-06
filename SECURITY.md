# Security Policy

Last updated: 2026-03-14

We use coordinated disclosure. Please report security issues privately.

This repository is explicit about one important trust boundary:

- the runtime can execute installed plugins
- plugins are not sandboxed
- installing a plugin is a trust decision, not a security boundary

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
- Email: [mousavi.bijan@gmail.com](mailto:mousavi.bijan@gmail.com)

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
- acknowledgement within 3 business days
- triage/update within 7 business days

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
