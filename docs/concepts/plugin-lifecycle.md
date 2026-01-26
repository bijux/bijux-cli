# Plugin Lifecycle

## Purpose
This document defines the plugin lifecycle contract for bijux-cli. It specifies states, transitions, and guarantees so plugin authors and operators can rely on predictable behavior.

## Scope
It covers discovery, installation, activation, deactivation, and removal. It does not explain plugin implementation details or the CLI command syntax for plugin management.

## Core Concepts
Plugins move through explicit lifecycle states: discovered, installed, active, inactive, and removed. Each transition is validated and must leave the registry and filesystem consistent.

## How to Think About This
Treat plugin lifecycle as a transactional workflow. A transition either completes and is recorded, or fails and is rolled back. This protects the system from partial installs and ambiguous state.

## Invariants
- Invalid metadata fails before activation.
- Registry state matches filesystem state after every transition.
- Activation never occurs without successful installation.

## Failure Modes
If a transition fails, the system rolls back any partial state and emits a stable error. The CLI does not leave half-installed or half-removed plugins in the registry.

## Design Rationale
Explicit lifecycle states reduce ambiguity and protect against corruption. They also make it possible to test lifecycle behavior deterministically.

## Non-Goals
This document does not specify plugin API design or compatibility beyond the metadata contract.

## References
- Implementation: `src/bijux_cli/plugins/`
- Regression coverage: `tests/regression/test_plugin_metadata_regression.py`
