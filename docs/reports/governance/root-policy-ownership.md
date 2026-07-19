# Root Policy Ownership

This report maps cross-package repository claims to their machine-readable
authority and enforcing crate. The complete inventory is
`contracts/foundation/root_policy_surface_inventory.v1.json`; this page is a
review route, not a second policy source.

## Governed Surfaces

| Repository question | Governing contract | Owning crate | Enforced meaning |
| --- | --- | --- | --- |
| Which maintainer commands are visible? | `contracts/foundation/maintainer_command_surface.v1.json` | `bijux-dev` | stable `bijux-dev-dag` root command inventory |
| How is repository work routed? | `contracts/foundation/repository_work_routing.v1.json` | `bijux-dev` | work classes, owning crates, allowed dependencies, and evidence roots |
| Which workspace crates are public? | `contracts/foundation/workspace_package_boundary.v1.json` | `bijux-dev` | publication intent for every workspace package |
| Which DAG command lanes ship? | `contracts/foundation/dag_release_truth_table.v1.json` | `bijux-dev` | stable, experimental, simulated, internal, and refused surfaces |
| Which product namespaces are reserved? | `contracts/official_product_namespace_registry.json` | `bijux-cli` | mounted-product ownership and collision policy |

## Review Route

When a root claim appears inconsistent:

1. identify the exact claim in a handbook, workflow, release note, or command;
2. locate its governing contract in the root policy inventory;
3. inspect the enforcing test and owning crate;
4. determine whether behavior, contract data, retained evidence, or
   explanation has drifted;
5. update every affected surface without weakening the enforcing check.

The report does not make a contract true. Its purpose is to make the authority
and executable owner discoverable before a reviewer accepts a repository-wide
claim.

## Verification

The root inventory is enforced by
`crates/bijux-dev/tests/foundation_root_policy_surface_inventory_contracts.rs`.
The visible maintainer command mapping is additionally checked by
`crates/bijux-dev/tests/foundation_maintainer_command_surface_contracts.rs`.
