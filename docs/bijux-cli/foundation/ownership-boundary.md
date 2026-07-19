---
title: Ownership Boundary
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Ownership Boundary

Use this page when you need to answer a simple question honestly: does this
behavior belong to `bijux`, to `bijux-dag`, or to repository-maintainer
surfaces outside the CLI product?

`bijux-cli` owns the runtime behavior behind the visible `bijux` command. It
does not own every tool, contract, or release rule that happens to live in the
same repository.

## Start With The Boundary That Matters

| If the issue is about... | The first owner is... |
| --- | --- |
| how `bijux` parses commands, resolves routes, shapes output, or handles plugins | `bijux-cli` |
| DAG authoring, execution, replay, cache proofs, or retained run evidence | `bijux-dag` |
| repository diagnostics, release verification, governance reports, or maintainer-only gates | `bijux-dev` and repository handbooks |

## What `bijux-cli` Owns

- `crates/bijux-cli/src/interface/` command and REPL interaction layers
- `crates/bijux-cli/src/routing/` parser, route catalog, and registry logic
- `crates/bijux-cli/src/contracts/` typed CLI-facing contracts
- `crates/bijux-cli/src/features/` stateful runtime features (config, plugins, history)

## What `bijux-cli` Does Not Own

- DAG execution semantics implemented in DAG crates
- repository-level release governance beyond CLI package concerns
- maintainer-only gates and dev orchestration behavior in `bijux-dev`
- product websites or publication pipelines outside CLI handbook scope

## Reader Rules

- If a user can observe it through `bijux`, this handbook should explain it.
- If a change affects both `bijux` and another product lane, do not hide the
  shared claim in CLI-only pages.
- If a question turns into DAG workflow semantics, move to the DAG handbook
  rather than stretching the CLI story until it becomes vague.

## Where The Boundary Is Enforced

- architecture boundary checks under
  `crates/bijux-cli/tests/architecture/boundaries/`
- route and parser contract checks under `crates/bijux-cli/tests/routing/`
- integration suites under `crates/bijux-cli/tests/integration/`

## Continue Reading

- [Repository Fit](repository-fit.md) for how `bijux` sits beside the other
  products in `bijux-core`
- [CLI Interfaces](../interfaces/index.md) for the caller-facing contract
- [Integration Seams](../architecture/integration-seams.md) for the places
  where this boundary is intentionally crossed
