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

from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.services.plugins import get_plugins_dir


def info_plugin(
    name: str = typer.Argument(..., help="Plugin name"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
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

    fmt_lower = validate_common_flags(fmt, command, quiet)

    plug_dir = get_plugins_dir() / name
    if not (plug_dir.is_dir() and (plug_dir / "plugin.py").is_file()):
        emit_error_and_exit(
            f'Plugin "{name}" not found',
            code=1,
            failure="not_found",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    meta_file = plug_dir / "plugin.json"
    meta: dict[str, Any] = {}
    if meta_file.is_file():
        try:
            meta = json.loads(meta_file.read_text("utf-8"))
            if not meta.get("name"):
                raise ValueError("Missing required fields")
        except Exception as exc:
            emit_error_and_exit(
                f'Plugin "{name}" metadata is corrupt: {exc}',
                code=1,
                failure="metadata_corrupt",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )

    payload = {"name": name, "path": str(plug_dir), **meta}

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, payload),
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
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
