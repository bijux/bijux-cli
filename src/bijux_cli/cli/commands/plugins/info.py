# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins info` subcommand for the Bijux CLI.

This module contains the logic for displaying detailed metadata about a single
installed plugin. It locates the plugin by name, reads its `plugin.json`
manifest file, and presents the contents in a structured, machine-readable
format.

Output Contract:
    * Success: `{"name": str, "path": str, ... (plugin.json contents)}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": "...", "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: The plugin was not found, or its metadata file was corrupt.
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

from collections.abc import Mapping
import json
import platform
from typing import Any

import typer

from bijux_cli.cli.constants import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.emit import emit_error_and_exit
from bijux_cli.cli.output import new_run_command, resolve_command_config
from bijux_cli.cli.validation import ascii_safe
from bijux_cli.core.enums import LogLevel
from bijux_cli.plugins.metadata import get_plugin_metadata


def info_plugin(
    name: str = typer.Argument(..., help="Plugin name"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Shows detailed metadata for a specific installed plugin.

    This function locates an installed plugin by its directory name, parses its
    `plugin.json` manifest file, and emits the contents as a structured
    payload.

    Args:
        name (str): The case-sensitive name of the plugin to inspect.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "plugins info"

    effective, _, fmt_lower = resolve_command_config(
        command=command,
        quiet=quiet,
        verbose=verbose,
        log_level=log_level,
        fmt=fmt,
        pretty=pretty,
    )
    quiet = effective.quiet
    verbose = effective.verbose_level > 0
    debug = effective.log_level == LogLevel.DEBUG
    pretty = effective.pretty

    try:
        meta = get_plugin_metadata(name)
    except Exception as exc:
        emit_error_and_exit(
            str(exc),
            code=1,
            failure="metadata_error",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            debug=debug,
        )

    payload: dict[str, Any] = {
        "name": meta.name,
        "version": meta.version,
        "enabled": meta.enabled,
        "source": meta.source,
        "requires_cli": meta.requires_cli,
    }
    if meta.dist_name:
        payload["package"] = meta.dist_name
    if meta.path:
        payload["path"] = str(meta.path)
        meta_file = meta.path / "plugin.json"
        try:
            extra = json.loads(meta_file.read_text("utf-8"))
            if isinstance(extra, dict):
                payload.update(extra)
        except Exception as exc:
            emit_error_and_exit(
                f'Plugin "{name}" metadata is corrupt: {exc}',
                code=1,
                failure="metadata_corrupt",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=effective.include_runtime,
                debug=debug,
            )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, payload),
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )


def _build_payload(
    include_runtime: bool, payload: dict[str, Any]
) -> Mapping[str, object]:
    """Builds the final payload with optional runtime metadata.

    Args:
        include_runtime (bool): If True, adds Python and platform info to the
            payload.
        payload (dict[str, Any]): The base payload containing the plugin metadata.

    Returns:
        Mapping[str, object]: The final payload, potentially with added runtime
            details.
    """
    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
    return payload
