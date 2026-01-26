# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins info` subcommand for the Bijux CLI.

This module contains the logic for displaying detailed metadata about a single
installed plugin. It locates the plugin by name, reads its `plugin.json`
manifest file, and presents the contents in a structured, machine-readable
format.

Output Contract:
    * Success: `{"name": str, "path": str, ... (plugin.json contents)}`
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

from bijux_cli.cli.core.command import (
    ascii_safe,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
    OPT_FORMAT,
    OPT_LOG_LEVEL,
    OPT_PRETTY,
    OPT_QUIET,
)
from bijux_cli.cli.core.help_text import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
)
from bijux_cli.core.enums import ErrorType
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.core.precedence import current_execution_policy, resolve_exit_intent
from bijux_cli.plugins.metadata import get_plugin_metadata


def info_plugin(
    name: str = typer.Argument(..., help="Plugin name"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Shows detailed metadata for a specific installed plugin.

    This function locates an installed plugin by its directory name, parses its
    `plugin.json` manifest file, and emits the contents as a structured
    payload.

    Args:
        name (str): The case-sensitive name of the plugin to inspect.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "plugins info"

    effective = current_execution_policy()
    fmt_lower = validate_common_flags(
        fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
        log_level=effective.log_level,
    )
    quiet = effective.quiet
    pretty = effective.pretty

    try:
        meta = get_plugin_metadata(name)
    except Exception as exc:
        intent = resolve_exit_intent(
            message=str(exc),
            code=1,
            failure="metadata_error",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            error_type=ErrorType.INTERNAL,
            log_level=effective.log_level,
        )
        raise ExitIntentError(intent) from exc

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
            intent = resolve_exit_intent(
                message=f'Plugin "{name}" metadata is corrupt: {exc}',
                code=1,
                failure="metadata_corrupt",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=effective.include_runtime,
                error_type=ErrorType.INTERNAL,
                log_level=effective.log_level,
            )
            raise ExitIntentError(intent) from exc

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, payload),
        quiet=quiet,
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
