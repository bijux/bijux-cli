---
title: Root Policy Surface Report
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-08
---

# Root Policy Surface Report

This page summarizes the repository-level contracts that govern ownership,
release boundaries, maintainer entrypoints, and namespace control across
`bijux-core`.

Primary inventory: `contracts/foundation/root_policy_surface_inventory.v1.json`.

## Anchored policy files

| Surface | Contract | Owner | What it freezes |
| --- | --- | --- | --- |
| maintainer command surface | `contracts/foundation/maintainer_command_surface.v1.json` | `bijux-dev` | visible `bijux-dev-dag` root commands and their user-facing inventory |
| backlog routing | `contracts/foundation/backlog_issue_class_routing.v1.json` | `bijux-dev` | issue-class ownership and evidence locations for repository governance work |
| workspace publication boundary | `contracts/foundation/workspace_package_boundary.v1.json` | `bijux-dev` | public versus private crate publication intent |
| DAG release boundary | `contracts/foundation/dag_release_truth_table.v1.json` | `bijux-dev` | stable, experimental, simulated, internal, and refused operator surfaces |
| official product namespaces | `contracts/official_product_namespace_registry.json` | `bijux-cli` | reserved runtime namespaces, aliases, and package ownership |

## How To Read This Report

- Start here when the question is about who owns a root policy or which file
  acts as the governing source of truth.
- Move to [Command Surface](../../bijux-dev/operations/command-surface.md) when
  the question is specifically about maintainer entrypoints.
- Move to [Backlog Routing Ledger](backlog-routing-ledger.md) when the question
  is about issue-class routing and evidence coverage.
