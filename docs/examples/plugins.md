# Plugins

## Purpose
This document shows a minimal plugin lifecycle and explains the consequences.

## Scope
It covers install, list, info, and uninstall only.

## What problem this solves
Plugin examples often skip validation and cleanup.
This example shows the full lifecycle with explicit cleanup.

## Why you should care
If you install plugins in CI, you need a predictable lifecycle and no residue.

## What confusion this removes
It removes uncertainty about when a plugin is registered and removed.

## Guarantees
Bijux guarantees:
1. Invalid plugins never activate.
2. Registry state matches filesystem state.

## How to Think About This
Treat plugins as lifecycle-managed commands, not arbitrary code.

## Common Misunderstandings
- "Install means activate immediately." It does not.

## Execution
Setup:

```bash
bijux plugin install ./my_plugin
```

Command:

```bash
bijux plugin list
```

Output:

```json
{"plugins":["my_plugin"]}
```

Implication:

Listing plugins is a registry view, not a filesystem scan.
If a plugin is listed, it passed metadata checks.

Cleanup:

```bash
bijux plugin uninstall my_plugin
```

## Failure Modes
- Invalid metadata exits with code 2.

## Design Rationale
We deliberately chose a full lifecycle example to show cleanup.
Why not only install? It hides removal behavior.

## Non-Goals
- Plugin scaffolding details.
