# Plugins

## Purpose
This guide explains how to install, inspect, and remove plugins safely. It exists to prevent partial installs and registry inconsistencies by making the lifecycle steps explicit.

## Scope
It covers install, list, info, and uninstall workflows. It does not teach plugin authoring; use the reference and concept docs for lifecycle guarantees.

## Installation
Install a plugin by pointing to its package or local path. Installation validates metadata before activation so failures are caught early.

```bash
bijux plugin install ./my_plugin
```

## Listing and Inspecting
List all installed plugins to confirm registry state, then request detailed information for a specific plugin.

```bash
bijux plugin list
bijux plugin info my_plugin
```

## Removal
Uninstall removes the plugin and updates the registry in a single, consistent operation.

```bash
bijux plugin uninstall my_plugin
```

## Why This Matters
Plugin state must remain consistent with the filesystem. These commands are designed to keep the registry and disk in sync, even when failures occur.

## References
- [Lifecycle contract](../concepts/plugin-lifecycle.md)
- [Command list](../reference/commands.md)
