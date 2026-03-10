# `bijux-cli-core/src/bin/bijux-rs.rs` Branch Audit

## Scope
Audit date: 2026-03-09
Commit basis: current `main` at and after `0c9933a`.

## Classification

### Fully implemented
- `emitter_config(flags)`
  - Deterministic mapping from parsed global flags to output emitter configuration.
- `home_dir()`
  - Reads home directory from `HOME` environment.
- `env_map()`
  - Captures compatibility environment variables for payload/debug surfaces.
- `find_command_mut(command, path)`
  - Recursive command lookup for help rendering.
- `try_render_clap_help(argv)`
  - Handles clap-generated `--help` and `--version` display flows.

### Compatibility shim
- `render_command_help(path)`
  - Uses clap long help and appends explicit compatibility note for inspect paths.
- `run_argv(argv)`
  - Preserves legacy `bijux help ...` behavior and ensures trailing newline in output.

### Placeholder or partial
- `route_response(normalized_path)`
  - Contains route-specific business logic in bin crate and returns static placeholder payloads for multiple commands.
  - Includes fallback `unknown route` data payload instead of fully policy-mapped error envelope.
- `_ = registry.register_plugin_namespace("community")`
  - Hardcoded namespace registration for simple bootstrap coverage; not full plugin loading flow.
- `main()`
  - Process bootstrap plus rendering, but does not own final stream routing and explicit exit mapping contract.

## Required follow-up from this audit
- Move route-specific branching into `core` and route ownership to `routing/core` boundaries.
- Add a single `run_app()` entrypoint in `core` and delegate from `main.rs`.
- Keep `main.rs` limited to argv bootstrap, calling core, writing streams, and exiting.
