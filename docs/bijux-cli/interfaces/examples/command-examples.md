---
title: Examples
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-06
---

# Examples

## Root Runtime

```bash
bijux status
bijux doctor --bundle
bijux apps list
```

## Config

```bash
bijux config schema cli
bijux config docs dag
bijux config validate --profile dev
bijux config explain cli.log_level
```

## App Routing

```bash
bijux-dag --help
bijux apps which dag
bijux doctor dag
```

When a product ships its own public binary, that binary remains the
authoritative operator surface. For DAG, use the
[DAG release boundary](../../../bijux-dag/foundation/release-boundary.md),
which is backed by the machine-readable contract
`contracts/foundation/dag_release_truth_table.v1.json`. Use `bijux-dag ...`
for stable operator procedures and `bijux dag ...` when you intentionally want
root-managed discovery or delegation.

## Plugin Runtime

```bash
bijux plugins list
bijux plugins inspect sample
bijux plugins doctor
```

## Python Bridge

```bash
bijux doctor python
python -m bijux_cli_py version
```
