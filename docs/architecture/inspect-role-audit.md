# Inspect Role Audit

Date: 2026-03-09
Scope: tasks 301-303

## Current role in Rust

`inspect` is a built-in diagnostics surface implemented in Rust core and exposed through both:

- `inspect`
- `cli inspect`

Current responsibilities in Rust:

1. Expose route and namespace visibility for command ownership.
2. Surface compatibility alias rewrites used by routing normalization.
3. Surface plugin origin metadata and compatibility warnings.
4. Publish diagnostics contract schema metadata for machine consumers.

## Current role in Python

Python command captures and command-tree artifacts show `inspect` is part of diagnostics and operator observability workflows, but full command-level inspect payload captures are not currently present in the golden lock.

In practice, Python role overlap is:

1. Route and plugin diagnostics inspection.
2. Environment and command-surface introspection for debugging.

## Alignment decision

For baseline diagnostics parity, Rust `inspect` remains a first-class built-in diagnostics API and not a compatibility shim.

## Contract decision

Because inspect output is machine-visible and consumed in parity tests, explicit typed contracts are required.

Added contract types:

- `RouteSourceMetadata`
- `AliasRewrite`
- `InspectReport`

These are now exported by `bijux-cli` (`contracts` module) and covered by serde roundtrip tests.
