# `bijux-dev` Architecture

`bijux-dev` is the private repository control plane. It inspects product facts,
executes governed suites, and generates reviewable evidence without becoming a
product runtime or redefining product contracts.

## Binary Boundaries

| Binary | Primary role |
| --- | --- |
| `bijux-dev-cli` | structured repository, package, runtime, documentation, Python, release, and evidence reports |
| `bijux-dev-dag` | governed checks, contract suites, evidence verification, generators, and release validation |

The binaries may share models and infrastructure, but similar names do not
justify duplicate implementations. One authority owns each operation.

## Source Boundaries

| Area | Responsibility |
| --- | --- |
| `maintainer/cli` | report-oriented argument and dispatch surface |
| `maintainer/reports` | report composition from repository and product queries |
| `maintainer/contracts` | maintenance/status contract inventories and execution |
| `maintainer/suites` | report-side suite catalog and execution |
| `commands` | `bijux-dev-dag` command families and governed orchestration |
| `suites` | reusable suite identifiers, metadata, filters, and release flow |
| `repo` | repository root and layout ownership |
| `report` | shared report model and writing |
| `tooling` and `maintainer/infra` | process, Cargo, Git, filesystem, clock, and artifact adapters |

Product algorithms do not move into these areas because a maintainer command
needs to observe them.

## Dependency Direction

The package may depend on CLI and DAG product crates to query public facts and
test declared boundaries. Product crates must not depend on `bijux-dev`.
`bijux-dag-testkit` is development-only.

Organization standards are consumed from synchronized `bijux-std` content.
This package validates those files but does not rewrite or locally reinterpret
their policy.

## Effect Model

Every command is classified:

- inspection reads and reports;
- validation may write transient artifacts and returns enforced status;
- generation writes named governed outputs;
- mutation changes an explicit repository target.

The default is read-only. Any source or governed-output write must be visible
in command naming, help, implementation, and tests.

## Runtime Boundaries

Repository roots and output paths are explicit. Local run products go under
`artifacts/`. Process execution passes through named adapters so tests can
verify arguments, status, and streams. Product state is queried through public
facades instead of private source traversal when an API exists.

## Extension Decisions

- Put product behavior in the product crate.
- Put reusable selection in suites, not Make recipes.
- Give each governed generator one discoverable owner.
- Use schemas for machine reports and typed models for internal composition.
- Preserve component failures in aggregate execution.
- Add a command only when its effect and authority are unambiguous.

## Verification

Architecture layout, depth, ownership, dependency, command-surface, and root
policy contracts protect these boundaries.
