---
title: Command Examples
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Command Examples

These examples show the shortest supported route to common `bijux` outcomes.
They are deliberately read-oriented until a command is explicitly described as
mutating state. Add `--format json --no-pretty` when a script needs a stable
machine-readable envelope rather than terminal presentation.

## Establish Runtime Health

```bash
bijux status
bijux apps list
bijux doctor
```

`status` summarizes the root runtime, `apps list` reports mounted product
namespaces, and `doctor` evaluates the local installation. None of these
commands repairs state. When a fault depends on machine state, retain a support
bundle under the command-selected artifact path:

```bash
bijux doctor --bundle
```

Review the emitted path before sharing it. A support bundle is diagnostic
evidence, not proof that every mounted product or plugin is healthy.

## Explain Configuration

```bash
bijux config schema cli
bijux config docs dag
bijux config validate --profile dev
bijux config explain cli.log_level
```

Use `schema` to inspect accepted keys and types, `docs` for generated reference
material, `validate` to check the selected profile, and `explain` to identify
the winning value and source. Prefer `explain` before editing configuration;
the visible file may be overridden by a profile, environment value, or command
argument.

For automation:

```bash
bijux config explain cli.log_level --format json --no-pretty
bijux config validate --profile dev --format json --no-pretty
```

A non-zero validation status is the contract. Do not infer success from a
partially populated JSON object.

## Resolve A Mounted Application

```bash
bijux apps which dag
bijux doctor dag
bijux-dag --help
```

`apps which` reports the entrypoint selected by root routing. `doctor dag`
checks integration from the root runtime. Neither command replaces the
product's own diagnostics.

When a product ships a public binary, that binary is the authoritative operator
surface. For DAG, use the
[DAG release boundary](../../bijux-dag/foundation/release-boundary.md),
which is backed by the machine-readable contract
`contracts/foundation/dag_release_truth_table.v1.json`. Use `bijux-dag ...`
for stable operator procedures and `bijux dag ...` when you intentionally want
root-managed discovery or delegation.

## Inspect Plugin State

```bash
bijux plugins list
bijux plugins inspect sample
bijux plugins doctor
```

`list` inventories registry records, `inspect` explains one record, and
`doctor` checks installed records against runtime expectations. These commands
do not execute an untrusted plugin merely to prove that its registry metadata
is valid. Plugin execution remains a trust boundary; inspect the manifest and
entrypoint before invoking third-party code.

## Verify The Python Distribution

```bash
bijux doctor python
python -m bijux_cli_py version
```

The Python module and the installed `bijux` console script resolve the same
Python launcher authority. `doctor python` checks root-to-Python integration;
the module invocation isolates packaging and interpreter discovery. The Python
distribution does not embed or install `bijux-dag`.

## Choose The Next Guide

| Observation | Continue with |
| --- | --- |
| a configuration value is surprising | [Configuration Surface](configuration-surface.md) |
| a mounted application resolves incorrectly | [App Integration Guide](app-integration-guide.md) |
| a plugin record is invalid or unsafe | [Diagnostics Guide](../operations/diagnostics-guide.md) |
| a command failed after changing state | [Failure Recovery](../operations/failure-recovery.md) |
| a script depends on output fields | [Data Contracts](data-contracts.md) |
