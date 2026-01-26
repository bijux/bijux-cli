# Plugin lifecycle

## Purpose
This document tells you when a plugin can load and what must be validated first.

## Scope
It covers discovery, install, activation, deactivation, and removal.

## What problem this solves
Plugins can corrupt state if they load with bad metadata or fail mid-activation.
This contract prevents partial activation and hidden residue.

## Why you should care
If you extend the CLI, you need to know exactly when a plugin is allowed to run.

## What confusion this removes
It removes ambiguity about when metadata is checked and how rollback works.

## Guarantees
Bijux guarantees:
1. Invalid metadata never reaches activation.
2. Registry and filesystem remain consistent after failures.
3. Activation and deactivation are symmetric.

## How to Think About This
Treat plugin lifecycle as a state machine with explicit transitions.
If a transition fails, rollback must restore the previous state.

## Common Misunderstandings
- "Plugins can activate before metadata validation." They cannot.
- "Uninstall can leave files behind." It must not.

## Lifecycle
- Discover: find candidates and validate metadata.
- Install: record plugin and place files.
- Activate: load code and register commands.
- Deactivate: remove commands and release resources.
- Remove: delete records and files.

## Failure Modes
- Missing metadata: reject before activation.
- Compatibility mismatch: reject before activation.
- Import failure: activation fails and registry is rolled back.
- Uninstall failure: registry entry remains and exit code 1 is returned.

## Design Rationale
We deliberately chose early validation to avoid partial activation.
Why not validate on first use? It hides failures until runtime.

## Non-Goals
- Plugin auto-update.
- Plugin sandboxing.

## References
- Implementation: `src/bijux_cli/plugins/registry.py`
- Regression coverage: `tests/regression/test_plugin_loader_regression.py`
