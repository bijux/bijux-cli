---
title: Known Limitations
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Known Limitations

This page records boundaries in the shipped CLI, not speculative risks or
failing tests. When another page describes a broader capability, the boundary
here takes precedence until its removal condition is met.

## Current Boundary Summary

| Area | Current boundary | Immediate consequence |
| --- | --- | --- |
| plugin isolation | plugins execute with the current user's OS authority | installing a plugin is a code-execution trust decision |
| plugin installation | local plugin source remains in place; it is not copied into a managed store | moving or deleting the manifest or entrypoint breaks the installed route |
| plugin runtimes | Python 3.11+, delegated Python, and external executables are supported; native plugins are not | a valid-looking native manifest is rejected |
| product delegation | official product routes coordinate separately installed runtimes | a known route may still fail when its binary or interpreter is unavailable |
| config text | keys and values are ASCII-only, and values are single-line | localized or multiline values cannot be stored directly |
| secret detection | redaction follows schema metadata and secret-like key names | a secret under an ordinary key name may be displayed |

## Plugins Are Not Isolated

Python and external-executable plugins can access the files, network, and
credentials available to the CLI process. Environment filtering and timeout
handling reduce accidental exposure and indefinite waits, but they are not a
sandbox.

**Workaround:** run untrusted code under an OS account or container with
restricted filesystem, network, and credential access.

**Removal evidence:** a documented isolation model with adversarial tests for
filesystem, network, environment, resource, and subprocess boundaries. A trust
label or manifest checksum is not sufficient.

See [Security and Safety](../operations/security-and-safety.md) for the enforced
process policy.

## Local Plugin Installation Is Source-Linked

Installation validates a manifest and writes a registry record containing its
source path and checksum. It does not copy the manifest, entrypoint, or
dependencies into a CLI-owned package store. Local entrypoint resolution is
anchored to the manifest directory, and load diagnostics report a missing or
changed manifest.

**Workaround:** keep each installed plugin in a stable, operator-owned
directory, use the canonical `plugin.manifest.json` filename, and uninstall
before relocating it.

**Removal evidence:** an install-store contract covering copy semantics,
dependency ownership, updates, rollback, integrity scope, and uninstall, with
cross-platform lifecycle tests.

## Runtime Kinds Are Deliberately Narrow

The runtime accepts Python/delegated entrypoints and external executables.
Python entrypoints require a discoverable Python 3.11 or newer interpreter.
The `native` manifest kind is represented in the schema but rejected during
validation and execution; there is no stable native plugin ABI.

**Workaround:** expose compiled extensions as `external-exec` plugins, or use a
Python entrypoint when Python is an acceptable runtime dependency.

**Removal evidence:** a versioned native ABI, loader safety model,
compatibility policy, release packaging contract, and host/plugin matrix tests.

## Official Products Remain Separate Installations

The root CLI knows official product namespaces and can resolve embedded,
binary, or Python entrypoints. It does not bundle every product runtime. When a
delegated command is unavailable, the CLI reports the failed command and the
applicable install command; route recognition alone does not guarantee local
availability.

**Workaround:** run `bijux doctor`, install the named product package, and make
one canonical runtime visible on `PATH`.

**Removal evidence:** either a distribution that owns the complete runtime set
or an explicit dependency resolver with install, version, provenance, and
rollback guarantees.

## Configuration Text Is ASCII And Single-Line

Configuration keys reject non-ASCII characters and punctuation outside the
supported key syntax. Values reject non-ASCII text and carriage return,
newline, tab, vertical-tab, and form-feed characters. This keeps file and
terminal behavior deterministic, but it excludes localized and multiline
values.

**Workaround:** store a path or identifier in CLI configuration and keep rich
content in a separately governed file.

**Removal evidence:** encoding and escaping rules that round-trip across
environment, file, flag, JSON, and table surfaces, with compatibility tests for
existing state.

## Secret Redaction Depends On Classification

Layered reports redact schema fields marked sensitive and keys whose names
contain recognized secret terms. The CLI does not inspect arbitrary values to
decide whether their content is secret. `--include-secrets` also disables the
display protection intentionally.

**Workaround:** use schema-defined sensitive fields or secret-bearing key names,
review output before publishing it, and never use `--include-secrets` in shared
logs.

**Removal evidence:** a stronger secret-classification contract that remains
deterministic, avoids unsafe content heuristics, and is covered across every
output mode.

## Implementation Evidence

- `crates/bijux-cli/src/features/plugins/runtime.rs` defines supported plugin
  execution and process policy.
- `crates/bijux-cli/src/features/plugins/registry.rs` records source-linked
  installation state.
- `crates/bijux-cli/src/features/plugins/entrypoint.rs` resolves local
  entrypoints relative to manifest provenance.
- `crates/bijux-cli/src/interface/cli/dispatch/delegation.rs` launches official
  product runtimes and reports missing installations.
- `crates/bijux-cli/src/features/config/validation.rs` enforces config text
  restrictions.
- `crates/bijux-cli/src/features/config/schema.rs` owns secret classification
  and redaction.

## Continue Reading

- [Risk Register](risk-register.md)
- [Security and Safety](../operations/security-and-safety.md)
- [Architecture Risks](../architecture/architecture-risks.md)
