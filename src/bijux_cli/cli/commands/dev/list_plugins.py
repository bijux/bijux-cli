# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `dev list-plugins` subcommand for the Bijux CLI.

This module provides a developer-focused command to list all installed CLI
plugins. It delegates its core logic to the shared `handle_list_plugins`
utility, which scans the filesystem and returns a structured list.

Output Contract:
    * Success: `{"plugins": [str, ...]}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An error occurred while accessing the plugins directory.
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.core.command import new_run_command, validate_common_flags
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
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.plugins.catalog import list_installed_plugins


def dev_list_plugins(
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Lists all installed CLI plugins.

    This command acts as a wrapper around the shared `handle_list_plugins`
    utility to provide a consistent interface for developers.

    Args:
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        log_level (str): The requested logging level.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "dev list-plugins"

    effective = current_execution_policy()
    validate_common_flags(
        fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
        log_level=effective.log_level,
    )
    plugins = list_installed_plugins()

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Build the list-plugins payload with optional runtime metadata."""
        payload: dict[str, object] = {"plugins": plugins}
        if include_runtime:
            payload.update(
                {
                    "python": platform.python_version(),
                    "platform": platform.platform(),
                }
            )
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=effective.quiet,
        fmt=effective.output_format,
        pretty=effective.pretty,
        log_level=effective.log_level,
    )
