# Plugins

## Purpose
This document guarantees a minimal plugin lifecycle example with consequences.

## Scope
It covers install, list, info, and uninstall only.

## Core Concepts
- Plugin metadata is validated before activation.

## Invariants
- Invalid plugins never activate.

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

- Plugin registry stays consistent with filesystem state.

Cleanup:

```bash
bijux plugin uninstall my_plugin
```

## Failure Modes
- Invalid metadata exits with code 2.

## Design Rationale
- Alternatives: template-only examples.
- Rejected because they skip real lifecycle.

## Non-Goals
- Plugin scaffolding details.
