---
title: App Integration Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# App Integration Guide

Mounted apps let `bijux` delegate a root namespace to another product or
application. The mount describes discovery and execution; it does not transfer
ownership of the mounted command surface to `bijux`.

## Choose The Integration Boundary

| Need | Use |
| --- | --- |
| ship an official Bijux product | add it to the governed official product registry and compile its descriptor into the host |
| replace or disable an official product for one environment | provide a project, configured, user, or system mount override |
| route a non-official external process | provide a complete mount descriptor for a custom namespace |
| test an in-process Rust app | implement `BijuxApp` and mount it in `BijuxCliHarness` |
| author a Python process app | use `bijux_cli_py.app_sdk` to build the descriptor and root-compatible output envelopes |
| extend `bijux` as an independently installed plugin | use the plugin lifecycle, not an official-product mount |

Official apps and plugins may share root routing and diagnostic conventions,
but they do not share trust, compatibility, or release ownership.

## Full Descriptor Contract

Ask the installed runtime for its exact schema:

```bash
bijux apps schema --json
bijux apps validate-manifest ./.bijux/apps/sample.mount.json --json
```

A complete `ProductMountDescriptor` requires:

- a canonical, non-reserved `namespace`;
- a display name and help summary;
- a runtime `entrypoint` and a `control_entrypoint`;
- zero or more aliases and capabilities;
- an optional app version;
- an optional semver compatibility window with an inclusive minimum and
  exclusive maximum host CLI version.

For example:

```json
{
  "namespace": "sample",
  "display_name": "Sample",
  "aliases": ["sample-tools"],
  "entrypoint": {
    "kind": "python_module",
    "command": "sample_app.cli",
    "module": "sample_app.cli",
    "function": "main"
  },
  "control_entrypoint": {
    "kind": "python_module",
    "command": "sample_app.cli",
    "module": "sample_app.cli",
    "function": "main"
  },
  "help": {
    "summary": "Sample project workflows"
  },
  "capabilities": ["json_output"],
  "version": "0.1.0",
  "compatibility": {
    "min_cli_version": "0.4.0",
    "max_cli_version_exclusive": "1.0.0"
  }
}
```

Namespaces and aliases cannot collide with root commands. Aliases cannot repeat
the canonical namespace, and capabilities are normalized and deduplicated.
Python `module` and `function` fields are valid only for `python_module`
entrypoints.

## Entrypoint Behavior

| Kind | Runtime behavior |
| --- | --- |
| `binary` | resolve the command through the environment and launch it as a child process |
| `python_module` | resolve a Python interpreter, then use `python -m module` or invoke the named callable |
| `python_console_script` | resolve and launch the installed console command |
| `plugin_process` | launch the declared process using the mounted-app process adapter |
| `embedded_rust` | dispatch to a handler compiled into the current Rust process |

External commands and Python code run with the invoking user's privileges.
Mounted apps are not sandboxed and may access the filesystem, environment,
network, credentials, and subprocess APIs available to that user. Descriptor
validation establishes routing integrity, not publisher identity or code
confinement.

## Discovery And Overrides

Official products begin with the compiled descriptor from
`contracts/official_product_namespace_registry.json`. The host then considers
descriptor files in this order:

1. the current project's `.bijux/apps/`;
2. directories from `BIJUX_APP_PATH`;
3. the user's `~/.bijux/apps/`;
4. the configured or default system app directory;
5. command resolution through `PATH`.

The first matching descriptor file wins. A full descriptor may replace the
compiled descriptor for that namespace. A partial official-product override
may replace display metadata, aliases, runtime and control entrypoints, help,
capabilities, or version, or set `disabled`. Its namespace must still match the
official namespace. Compatibility is part of a full descriptor rather than
the partial override shape.

For a custom namespace, discovery uses the project, configured, user, and
system descriptor directories and requires a complete valid descriptor.
Relative process entrypoints containing a path are resolved against the
descriptor's directory.

`disabled.json` files can disable official or custom namespaces. An invalid
higher-precedence official override is reported as `bad_manifest`; it is not
silently skipped in favor of a lower-precedence file.

## Official Inventory Is Not Custom Registration

These commands report compiled official products and their overrides:

```bash
bijux apps list --json
bijux apps which dag --json
bijux apps doctor dag --json
bijux apps version dag --json
bijux apps capabilities dag --json
```

A custom descriptor can be validated and invoked through its namespace, but it
does not become an official product merely because it routes successfully.
Do not use the official inventory commands as proof that every custom mount is
registered.

When an installed plugin claims an official product namespace, the official
app remains authoritative and diagnostics expose the shadowed plugin conflict.
Resolve the plugin registry rather than changing official ownership to make the
conflict disappear.

## Rust Integration

Use `ProductMount` to build and validate a descriptor, `BijuxApp` to implement
an in-process route, and `BijuxCliHarness` to prove aliases, compatibility,
output streams, and structured failures without launching the root binary.

The harness is a contract-test surface. Implementing `BijuxApp` in a test does
not automatically register that app in the installed `bijux` executable.
Production embedded handlers must be compiled into the owning distribution.

## Python Integration

`bijux_cli_py.app_sdk` provides:

- `build_python_mount_manifest` for a full Python descriptor;
- `CompatibilityWindow` and `compatibility_report` for host-version checks;
- `success` and `failure` for root-compatible envelopes;
- `run_json_app` for callable execution with application logs redirected to
  stderr.

Reserve stdout for successful structured output and stderr for diagnostics and
structured failures. `run_json_app` converts uncaught exceptions into a
`python_app_exception` failure, but application authors should still use
stable domain-specific error codes for expected failures.

Generate a runnable starting point when appropriate:

```bash
bijux apps scaffold python sample-python --path ./sample-python-app
bijux apps scaffold rust sample-rust --path ./sample-rust-app
```

Review generated dependencies, compatibility floors, and executable behavior
before publishing. A scaffold is a starting contract, not release evidence.

## Authorities And Tests

- descriptor types and validation:
  `crates/bijux-cli/src/contracts/product_mount.rs`
- discovery, override, diagnostics, and scaffolding:
  `crates/bijux-cli/src/features/apps.rs`
- Rust SDK and harness: `crates/bijux-cli/src/sdk/`
- Python SDK:
  `crates/bijux-cli-python/python/bijux_cli_py/app_sdk.py`
- SDK contracts:
  `crates/bijux-cli/tests/routing/contracts/sdk_surface.rs`
- process-level app coverage:
  `crates/bijux-cli/tests/integration/cli/root/apps_command_coverage.rs`

## Continue Reading

- [App Integration Scenario](app-integration-scenario.md)
- [Command Surface](cli-surface.md)
- [Security And Safety](../operations/security-and-safety.md)
- [Compatibility Commitments](compatibility-commitments.md)
