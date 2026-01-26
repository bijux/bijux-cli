# Plugins

## Purpose
This document tells you how to install and remove plugins safely.

## Scope
It covers install, list, info, and uninstall only.

## What problem this solves
Unsafe plugin installs can leave residue and corrupt registry state.
This guide keeps the lifecycle explicit.

## Why you should care
A predictable lifecycle prevents broken CI and manual cleanup.

## What confusion this removes
It removes ambiguity about when a plugin is registered and removed.

## Guarantees
Bijux guarantees:
1. Invalid metadata fails before activation.
2. Registry and filesystem remain consistent after failures.

## How to Think About This
Treat plugins as lifecycle-managed commands, not arbitrary code.

## Common Misunderstandings
- "Install means activate immediately." It does not.

## Execution
```bash
bijux plugin install ./my_plugin
bijux plugin list
bijux plugin info my_plugin
bijux plugin uninstall my_plugin
```

## Failure Modes
- Missing metadata exits with code 2.
- Uninstall failures exit with code 1.

## Design Rationale
We deliberately chose early validation to prevent partial activation.
Why not validate on first use? It delays failures into production.

## Non-Goals
- Plugin auto-update.

## References
- Lifecycle contract: `concepts/plugin-lifecycle.md`
- Commands: `reference/commands.md`
