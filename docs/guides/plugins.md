# Plugins

## Purpose
This document guarantees a minimal plugin lifecycle workflow.

## Scope
It covers install, list, info, and uninstall only.

## Core Concepts
- Plugins are validated before activation.
- Plugin metadata controls compatibility.

## Invariants
- Invalid metadata fails before activation.
- Registry and filesystem stay consistent after failures.

## Execution
```bash
bijux plugin install ./my_plugin
bijux plugin list
bijux plugin info my_plugin
bijux plugin uninstall my_plugin
```

## Failure Modes
- Missing metadata exits with code 2.
- Incompatible plugins exit with code 2.
- Uninstall failures exit with code 1.

## Design Rationale
- Alternatives: auto-activation without validation.
- Rejected because it risks partial loads.

## Non-Goals
- Plugin auto-update.

## References
- Lifecycle contract: `concepts/plugin-lifecycle.md`
- Commands: `reference/commands.md`
