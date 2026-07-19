---
title: Config Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
---

# Config Guide

`bijux-cli` now treats configuration as a layered, typed surface instead of a
flat env file only.

## Resolution Order

1. global env-backed config file
2. global named profile
3. project `.bijux/config.toml` or `.bijux/config.json`
4. project profile overlay
5. environment-variable overrides

Later layers win.

## Operator Commands

- `bijux config schema [scope]`
- `bijux config docs [scope]`
- `bijux config validate [--profile <name>]`
- `bijux config explain <key>`
- `bijux config repair`
- `bijux config export/load --portable`

## Secret Handling

Sensitive keys are redacted by default in explain and docs/reporting surfaces.
Use `--include-secrets` only when the caller explicitly needs raw values.

## Generated Reference

The checked-in reference lives at
[`config-generated-reference.md`](config-generated-reference.md) and is derived
from the same schema registry the runtime exposes.
