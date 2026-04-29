---
title: Examples
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
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
bijux dag --help
bijux apps which dag
bijux doctor dag
```

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
