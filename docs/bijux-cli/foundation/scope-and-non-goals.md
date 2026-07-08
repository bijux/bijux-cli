---
title: Scope and Non-Goals
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Scope and Non-Goals

Use this page when you need the honest version of the CLI story: what `bijux`
is expected to do well today, and what it does not claim to solve yet.

The goal is not to sound smaller than the product really is. The goal is to
keep readers from mistaking adjacent tooling, future ambitions, or unsafe
assumptions for the current CLI contract.

## What `bijux` Is For

`bijux` is the operator-facing command runtime in this repository. It is built
to parse commands predictably, execute built-in runtime features, mount apps,
load plugins, and return stable output and exit behavior that automation can
trust.

## In Scope

- deterministic parsing and normalization of command paths
- runtime policy resolution from flags, config, and defaults
- built-in command execution (`status`, `audit`, `docs`, `config`, `plugins`, and more)
- plugin discovery, manifest checks, route registration, and lifecycle toggles
- stable error shaping for usage, validation, and internal runtime failures

## What Readers Should Not Assume

- Plugin execution is a convenience and extensibility surface, not a hardened
  sandbox.
- The repository does not yet promise a stable in-process extension ABI for all
  host integrations.
- The CLI handbook does not absorb DAG semantics, maintainer gates, or every
  repository-wide workflow just because they live beside `bijux`.

## Non-Goals

- treating plugin execution as a hardened security sandbox
- promising a stable in-process extension ABI at current maturity
- collapsing all repository behavior into the root handbook
- claiming Windows host support as a complete product contract

## Practical Reading Rule

- Stay inside the CLI handbook when the issue is visible through `bijux`.
- Move to the DAG handbook when the real question is graph execution or replay.
- Move to maintainer and repository docs when the real question is release
  proof, governance, or cross-product workflow.

## Code Anchors

- `crates/bijux-cli/src/routing/model.rs`
- `crates/bijux-cli/src/routing/registry.rs`
- `crates/bijux-cli/src/features/plugins/`
- `crates/bijux-cli/src/interface/cli/dispatch/policy.rs`
- `crates/bijux-cli/src/kernel/`

## Review Focus

If behavior affects command compatibility, route identity, output payload shape,
or exit semantics, it belongs in scope and must be reviewed as a contract
change. If it is internal convenience with no external contract effect, it
belongs in implementation detail.

## Continue Reading

- [Ownership Boundary](ownership-boundary.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
- [Known Limitations](../quality/known-limitations.md)
