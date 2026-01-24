# Plugin Lifecycle

This document defines the expected lifecycle for Bijux CLI plugins:

install → verify → list → info → check → load → unload → uninstall

## Install

Inputs:
- Package name (PyPI name, ASCII letters/digits/dash/underscore)
- Environment: `PIP_FIND_LINKS`, `PIP_NO_INDEX`, `PIP_DISABLE_PIP_VERSION_CHECK`

Outputs:
- Success (stdout, JSON/YAML): `{"status": "installed", "package": "<name>", "plugins": ["<plugin>", ...]}`
- Dry run: `{"status": "dry-run", "package": "<name>"}`

Failure modes:
- Invalid name or non-ASCII input
- `pip install` failure (network, wheel not found, build failure)
- Missing `bijux_cli.plugins` entry point in the installed package
- Incompatible `bijux-cli` version requirement

## Verify

Definition:
- Metadata validation for each discovered plugin:
  - Entry-point plugins: validate `Requires-Dist` includes `bijux-cli` and that the
    current CLI version satisfies the requirement.
  - Local plugins: validate `plugin.json` fields and version compatibility.

Inputs:
- Plugin entry points and/or local plugin directory contents

Outputs:
- Internal validation result used by discovery; no direct CLI output

Failure modes:
- Invalid or missing metadata (e.g., `plugin.json` missing fields)
- Duplicate plugin names
- Version spec parse errors or unmet compatibility

## List

Inputs:
- `plugins list` (optional `--format`, `--pretty`, `--quiet`)

Outputs:
- Success: `{"plugins": [{"name": "...", "version": "...", "enabled": true}, ...]}`

Failure modes:
- Plugins dir inaccessible or a symlink
- Invalid format flag

## Info

Inputs:
- `plugins info <name>` (optional format flags)

Outputs:
- Success: `{"name": "...", "version": "...", "enabled": true, "source": "...", "requires_cli": "...", ...}`
- For local plugins, merges `plugin.json` contents into the payload.

Failure modes:
- Plugin not found
- Corrupt or unreadable `plugin.json`
- Invalid format flag

## Check

Inputs:
- `plugins check <name>` (optional format flags)

Outputs:
- Healthy: `{"plugin": "<name>", "status": "healthy"}`
- Unhealthy: `{"plugin": "<name>", "status": "unhealthy"}` (exit code 1)

Failure modes:
- Plugin not found
- Plugin has no local `plugin.py` (entry-point-only plugins report
  `health_unavailable`)
- Import errors or invalid `health(di)` hook signature
- Invalid format flag

## Load

Definition:
- Dynamic import of plugin code and registration of commands into the CLI runtime.
  This occurs when the CLI boots and the registry loads discovered plugins.

Inputs:
- `plugin.py` (local) or entry-point module for the plugin

Outputs:
- Commands registered in the CLI runtime; no direct CLI output

Failure modes:
- Import errors
- Signature or runtime errors in plugin initialization

## Unload

Definition:
- Release of plugin resources and removal from the active registry. This is a
  runtime lifecycle step and does not have a CLI subcommand today.

Inputs:
- Plugin registry entries

Outputs:
- Plugin removed from active registry; no direct CLI output

Failure modes:
- Cleanup errors in plugin teardown (logged/telemetry only)

## Uninstall

Inputs:
- `plugins uninstall <name>` (optional format flags)

Outputs:
- Success: `{"status": "uninstalled", "plugin": "<name>"}`

Failure modes:
- Plugin not installed
- `pip uninstall` failure for entry-point plugins
- Permission or filesystem errors when removing local plugin directories
- Invalid format flag
