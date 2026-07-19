---
title: Maintainer Package
audience: maintainers
type: package-index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Maintainer Package

The maintainer product has one private workspace package:
[`bijux-dev`](bijux-dev.md). Keeping one package is deliberate. Repository
checks often need a common view of Cargo metadata, source layout, contracts,
and retained evidence; splitting those checks by historical initiative would
create multiple policy authorities.

## Ownership Boundary

| `bijux-dev` owns | Product crates own |
| --- | --- |
| repository policy evaluation | CLI and DAG runtime behavior |
| contract and documentation governance | public command semantics |
| evidence generation and verification | graph, execution, and artifact semantics |
| release-readiness diagnostics | Python bridge behavior |
| maintainer command and suite catalogs | product recovery and operator workflows |

The package may query public product contracts and inspect repository state. A
product crate must not depend on `bijux-dev`, and a maintainer report must not
become an alternate implementation of product truth.

## Command Authorities

The public maintainer entrypoints are governed rather than inferred from files
under `src/bin/`:

- `contracts/foundation/maintainer_command_surface.v1.json` owns the command
  classification and package boundary.
- `crates/bijux-dev/docs/CONTRACTS.md` states package invariants.
- `crates/bijux-dev/src/suites/` owns reusable suite composition.
- `crates/bijux-dev/src/commands/` owns maintainer command behavior.
- `crates/bijux-dev/tests/` checks command, policy, evidence, and ownership
  contracts.

## Add Or Place Maintainer Behavior

1. Put repository inspection or policy logic in an owned command domain.
2. Put reusable gate composition in the suite catalog rather than copying a
   shell command into several workflows.
3. Return structured evidence with an owner and source identity when a release
   decision consumes the result.
4. Add focused contract coverage and a handbook remediation route.
5. Keep local and hosted entrypoints delegated through the same make target.

Do not add a maintainer command when a library query, product test, or existing
suite can own the behavior more directly.

## Package Detail

Open [`bijux-dev`](bijux-dev.md) for source layout, review boundaries, and
package-local verification. Use [Command Surface](../operations/command-surface.md)
for executable entrypoints and [Repository Gates](../operations/repository-gates.md)
for choosing a verification lane.
