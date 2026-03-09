# Plugin Sandboxing Policy

`bijux-cli` enforces capability-first execution for plugins.

## Runtime Boundary

- Plugins execute outside core command handlers.
- `native` plugin kind is reserved and not executable in v1.
- `delegated`, `python`, and `external-exec` kinds must pass manifest and compatibility checks before activation.

## Capability Guard

- Plugin execution requires declared capabilities.
- Missing required capability is a hard runtime error.
- Capability checks happen before entrypoint resolution.

## Trust Levels

`bijux-cli` assigns trust metadata to each plugin:

- `core`: distributed with official Bijux products.
- `verified`: installed from signed/verified provenance.
- `community`: third-party plugin with explicit user opt-in.
- `unknown`: plugin provenance could not be validated.

Trust level is surfaced in plugin inspect and route introspection outputs.

## Load Safety

- Broken and incompatible plugins are diagnosed at load time.
- Corrupted registry files are quarantined before safe self-repair.
- Plugin load order is deterministic and stable across restarts.
