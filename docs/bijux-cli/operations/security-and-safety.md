---
title: Security and Safety
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Security and Safety

Use this page when the question is not whether the CLI is convenient, but
whether it is safe to trust in the way you are about to use it.

`bijux-cli` security posture is built around explicit trust boundaries, safe
configuration handling, and visible plugin lifecycle controls. It is not based
on pretending extension execution is more isolated than it really is.

## Safety Boundaries

- plugin installation is a trust decision, not a sandbox guarantee
- reserved namespaces prevent extension collisions with core/runtime roots
- config values are validated for ASCII and control-character safety
- diagnostics surface path conflicts and plugin health warnings

## Code Anchors

- `crates/bijux-cli/src/contracts/plugin.rs`
- `crates/bijux-cli/src/routing/registry.rs`
- `crates/bijux-cli/src/contracts/config.rs`
- `crates/bijux-cli/src/features/plugins/operations.rs`
- `crates/bijux-cli/src/interface/cli/handlers/cli.rs`

## What These Boundaries Protect

| Boundary | Why it matters |
| --- | --- |
| plugin trust is explicit | users should not mistake extensibility for isolation |
| reserved namespaces stay enforced | core and plugin routes must not become ambiguous |
| config validation stays strict | hostile or malformed values should fail early and visibly |
| diagnostics remain available | operators need evidence when safety assumptions are under stress |

## Safety Rules

- do not auto-trust external plugin manifests
- keep plugin trust and compatibility metadata visible in reports
- fail explicitly on invalid config and namespace conflicts
- keep diagnostics available for operator safety triage

## Reader Shortcut

If your safety story depends on plugin code behaving well by convention, the
real trust boundary is human judgment, not enforcement. This page is here to
keep that fact visible.

## Continue Reading

- [Extensibility Model](../architecture/extensibility-model.md)
- [Failure Recovery](failure-recovery.md)
- [Known Limitations](../quality/known-limitations.md)
