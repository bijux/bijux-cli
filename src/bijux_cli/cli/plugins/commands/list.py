# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins list` subcommand for the Bijux CLI.

This module provides the primary command for listing all installed CLI plugins.
It performs security checks on the plugins directory and then delegates its
core logic to the shared `handle_list_plugins` utility, which scans the
filesystem and returns a structured list.

Output Contract:
    * Success: `{"plugins": [{"name": str, "version": str, "enabled": bool}, ...]}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An error occurred while accessing the plugins directory (e.g.,
      it is a symlink or inaccessible).
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
from bijux_cli.cli.plugins.commands.validation import refuse_on_symlink
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.plugins import get_plugins_dir
from bijux_cli.plugins.catalog import list_installed_plugins


def list_plugin(
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Lists all installed CLI plugins.

    This command first performs security checks on the plugins directory, such
    as ensuring it is not a symbolic link. It then delegates to the shared
    `handle_list_plugins` utility to perform the filesystem scan and emit the
    structured output.

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
    command = "plugins list"

    effective = current_execution_policy()
    validate_common_flags(
        fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
        log_level=effective.log_level,
    )
    plugins_dir = get_plugins_dir()
    refuse_on_symlink(
        plugins_dir,
        command,
        effective.output_format,
        effective.quiet,
        effective.log_level,
    )
    plugins = list_installed_plugins()

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Build plugin list payload with optional runtime metadata."""
        payload: dict[str, object] = {"plugins": plugins}
        if include_runtime:
            payload["python"] = platform.python_version()
            payload["platform"] = platform.platform()
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=effective.quiet,
        fmt=effective.output_format,
        pretty=effective.pretty,
        log_level=effective.log_level,
    )
