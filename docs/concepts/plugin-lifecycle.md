# Plugin lifecycle

## Purpose
This document guarantees plugin state transitions and validation rules.

## Scope
This covers discovery, installation, activation, and removal within the CLI.

## Core Concepts
- States: discovered, installed, active, inactive, removed.
- Metadata validation happens before activation.

## Invariants
- Invalid metadata never reaches activation.
- Registry state and filesystem state remain consistent after failures.
- Activation and deactivation are symmetric operations.

## Lifecycle
### Discover
Finds candidate plugins and validates metadata.

### Install
Installs a plugin into the registry and filesystem.

### Activate
Loads plugin code and registers commands.

### Deactivate
Unloads plugin commands and releases resources.

### Remove
Deletes plugin records and files.

## Failure Modes
- Missing metadata: plugin rejected before activation.
- Compatibility mismatch: plugin rejected before activation.
- Import failure: activation fails and registry is rolled back.
- Uninstall failure: registry entry is preserved with error exit.

The CLI does not attempt recovery beyond rollback.

## Design Rationale
- Alternatives: lazy validation at first use.
- Rejected because it allows partial activation and inconsistent state.
- Chosen: validate early and fail fast.

## Non-Goals
- Plugin auto-update.
- Plugin sandboxing.

## References
- Implementation: `src/bijux_cli/plugins/registry.py`
- Regression coverage: `tests/regression/test_plugin_loader_regression.py`
