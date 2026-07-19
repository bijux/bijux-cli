# `bijux-dev` Contracts

`bijux-dev` is the private repository control plane. It turns checked-in
policy, suite catalogs, contracts, and generators into maintainer commands and
reviewable evidence. It is not an end-user product package.

## Binary Authorities

| Binary | Owned responsibility |
| --- | --- |
| `bijux-dev-cli` | repository diagnostics, maintenance audits, documentation publication commands, and structured maintainer reports |
| `bijux-dev-dag` | governed suite catalogs, DAG policy and contract execution, evidence verification, and release-proof composition |

`contracts/foundation/maintainer_command_surface.v1.json` governs the visible
`bijux-dev-dag` root command surface. Similar command names must delegate to
one implementation or document an intentional semantic boundary.

## Package Boundary

The package may:

- read product contracts, schemas, fixtures, and public APIs;
- depend on product crates to verify their declared behavior;
- generate governed reports from explicit repository inputs;
- orchestrate focused validation suites;
- inspect repository structure and release metadata;
- write transient evidence under `artifacts/`.

It must not:

- become an alternate implementation of `bijux` or `bijux-dag`;
- move product policy out of the owning product package merely for convenient
  inspection;
- let product crates depend on maintainer code;
- rewrite synchronized `bijux-std` content;
- report advisory, narrowed, simulated, or partial evidence as a blocking full
  result.

## Command Effect Contract

Every command has an honest effect class:

- read-only inspection does not mutate source or governed outputs;
- validation may write transient reports but leaves governed source unchanged;
- generation writes named governed outputs and is checked for drift;
- mutation identifies its target and requires an explicit command surface.

Commands direct local run products to `artifacts/`. A command that writes
`docs/spec`, `docs/reports`, contracts, or configuration is a governed
generator and must be documented and tested as such.

## Suite Contract

Suite selection records group, domain, slow/internal inclusion, disabled
entries, advisory state, and selected suite IDs. Required failures return
nonzero. Advisory mode may preserve findings without failing, but its result
cannot support a blocking claim.

Aggregate suites:

- retain each component failure;
- continue when their declared policy is non-fail-fast;
- preserve final nonzero status when required work failed;
- distinguish unselected work from passing work;
- emit enough evidence to reproduce the selected command.

## Report Contract

A governed report identifies or has a discoverable producer, input authority,
scope, and stale-output check. Generators must be deterministic for the same
source state or explain the intentionally variable field.

Reports observe product state; they do not redefine product contracts.
Generated output drift is reviewed semantically before commit.

## Make And CI Boundary

Make targets and workflows own environment setup and orchestration. They
delegate policy and suite behavior to this package or another named authority.
A wrapper preserves:

- exact selection and exclusions;
- command status through pipes and logging;
- aggregate results;
- artifact paths;
- pinned tools required by the command.

## Dependency Direction

`bijux-dev` may depend on CLI and DAG packages. Product packages must not
depend on it. `bijux-dag-testkit` is a development-only dependency.

Organization-wide standards are consumed from synchronized sources and remain
owned by `bijux-std`.

## Verification

| Change | Required evidence |
| --- | --- |
| command surface | maintainer command-surface and route contracts |
| suite selection or status | suite dispatch unit tests and control-plane contracts |
| report generator | focused generator test plus clean governed-output diff |
| docs governance | documentation source-reference and governance contracts |
| release policy | release-validation and package-boundary contracts |
| source ownership | architecture layout and ownership tests |

Run the narrow contract test for a bounded change. A broad control-plane claim
requires the package suite and the relevant Make or hosted wrapper evidence.

Changing a product contract requires product implementation and tests in the
owning crate. Changing a synchronized standard requires the upstream standards
workflow rather than a downstream exception.
