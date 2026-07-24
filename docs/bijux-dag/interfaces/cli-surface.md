---
title: CLI Surface
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-23
---

# CLI Surface

The `bijux-dag` command tree separates stable operator contracts from
experimental helpers, modeled simulations, and internal verification routes.
Release-lane classification determines discovery, execution controls, and
compatibility promises.

Use [Generated CLI Reference](generated-cli-reference.md) for the exact stable
commands, arguments, and flags generated from the binary. Use
[Gated Command Inventory](gated-command-inventory.md) for the generated
experimental, simulated, and internal tree.

## v0.4.0 Surface Truth Table

| Class | Compatibility meaning | Discovery and access |
| --- | --- | --- |
| stable | supported operator surface for local authoring, execution, replay, and evidence inspection | visible in `bijux-dag --help` and `bijux-dag commands` |
| experimental | repository-tested operator helpers outside the stable compatibility promise | callable by explicit path; inventory with `bijux-dag commands --lane experimental` |
| simulated | modeled platform and control-plane behavior, not a production backend or service | inventory with `--lane simulated`; execute only with `BIJUX_DAG_ENABLE_SIMULATED=1` |
| internal | maintainer and contract-verification routes outside the product API | inventory with `--lane internal`; execute only with `BIJUX_DAG_ENABLE_INTERNAL=1` |
| unreleased | capabilities that v0.4.0 does not promise | no supported command contract |

The canonical classification is
[Release Boundary](../foundation/release-boundary.md), backed by
`contracts/foundation/dag_release_truth_table.v1.json`. A command moving
between rows is a release-boundary change, not a documentation-only edit.

## Choose A Stable Route

| Operator intent | Start with | Continue with |
| --- | --- | --- |
| validate authored work | `validate`, `plan` | [Graph Schema](graph-schema.md) |
| execute or reproduce work | `run`, `replay`, `verify` | [Operator Workflows](operator-workflows.md) |
| inspect retained evidence | `runs`, `artifact`, `artifact-inspect`, `explain` | [Run Evidence Layout](run-evidence-layout.md) |
| compare outcomes | `diff` and retained-run comparison routes | [Reproducibility Model](reproducibility-model.md) |
| inspect local health or cache | `doctor`, `cache` | [Failure Recovery](../operations/failure-recovery.md) |
| discover the interface | `commands`, `version`, `completions` | [Generated CLI Reference](generated-cli-reference.md) |

Global output flags such as `--json` and `--quiet` are documented with their
owning commands in the generated reference. Scripts should use structured
output and machine-readable status fields rather than parsing human prose.

## Visible Root Surface

The stable root commands are:

- authoring and planning: `validate`, `plan`
- execution and verification: `run`, `replay`, `verify`
- retained evidence: `runs`, `artifact`, `artifact-inspect`, `diff`, `explain`
- local operation and discovery: `doctor`, `cache`, `version`, `commands`,
  `completions`

Stable does not mean every behavior beneath a command is equivalent. For
example, a plan preview is advisory, while a retained run can support evidence
claims. The command reference states accepted input; the owning contract page
states what the result proves.

## Hidden Experimental Routes

Experimental routes are callable by explicit path but remain absent from the
default root help and default catalog:

- graph helpers: `init`, `canonicalize`, `graph`, `graph-lint`, `fingerprint`,
  `hash`
- inspection helpers: `status`, `node`, `trace-artifact`, `why-rerun`,
  `why-cache-missed`
- bundle and policy helpers: `export`, `import`, `migrate`, `adapters`,
  `config`, `policy`, `fsck`, `prove`, `proof-summary`

These commands may have tests and useful behavior without carrying the stable
compatibility promise. Automation that depends on one must acknowledge that
release posture rather than presenting it as a stable operator API.

## Hidden Simulation And Maintainer Namespaces

Simulated root namespaces are `control-plane`, `state-store`, `dataset`,
`enterprise`, `fleet`, `governance`, `federation`, `incident`, and `lab`.
They model contracts and organizational workflows; they do not claim deployed
services or production backends.

Internal root namespaces are `security`, `durability`, `performance`,
`release`, `runtime`, `schedule`, `version-inspect`, `capabilities`,
`semantic-portability`, and `equivalence-proof`. They support repository
governance and contract verification, not public operator integrations.

Inventory and execution are separate controls:

- `bijux-dag commands --lane simulated` lists modeled routes;
- `BIJUX_DAG_ENABLE_SIMULATED=1` permits deliberate simulated execution;
- `bijux-dag commands --lane internal` lists maintainer routes;
- `BIJUX_DAG_ENABLE_INTERNAL=1` permits deliberate internal execution.

The environment variables do not promote a route into the stable lane.

## Contract Authorities

| Question | Owning page |
| --- | --- |
| What exact flags does a stable command accept? | [Generated CLI Reference](generated-cli-reference.md) |
| Which gated routes exist today? | [Gated Command Inventory](gated-command-inventory.md) |
| How do selection, path preview, or resource budgets behave? | generated reference plus [Operator Workflows](operator-workflows.md) |
| What evidence does a run retain? | [Run Evidence Layout](run-evidence-layout.md) |
| Why did replay or cache reuse succeed or refuse? | [Reproducibility Model](reproducibility-model.md) |
| Which capabilities are intentionally unsupported? | [Known Limitations](../quality/known-limitations.md) |

Generated references state what the binary accepts. Contract and workflow
authorities state what a result means and which evidence can support a
decision.

## Change Discipline

A command-surface change is complete only when:

- the machine-readable release lane matches the intended compatibility;
- Clap help and generated references are regenerated;
- routing and release-boundary contracts pass;
- operator guidance describes any new evidence or failure semantics;
- hidden routes remain absent from default discovery unless promotion is
  intentional.

Deprecation and removal require explicit compatibility treatment. Hiding a
route or renaming it in prose is not a substitute for governing the binary
surface.

## Code Anchors

- binary handoff: `crates/bijux-dag-cli/src/main.rs`
- command definitions: `crates/bijux-dag-app/src/commands/`
- release classification:
  `contracts/foundation/dag_release_truth_table.v1.json`
- generated references:
  `crates/bijux-dag-app/src/commands/reference_docs.rs`

## Operator Authorities

- [Generated CLI Reference](generated-cli-reference.md)
- [Operator Workflows](operator-workflows.md)
- [Entrypoints And Examples](entrypoints-and-examples.md)
- [Release Boundary](../foundation/release-boundary.md)
