# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `dev list-plugins` subcommand for the Bijux CLI.

This module provides a developer-focused command to list all installed CLI
plugins. It delegates its core logic to the shared `handle_list_plugins`
utility, which scans the filesystem and returns a structured list.

Output Contract:
    * Success: `{"plugins": [str, ...]}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
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

from bijux_cli.cli.constants import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.output import new_run_command, resolve_command_config
from bijux_cli.cli.validation import validate_common_flags
from bijux_cli.services.plugins.listing import list_installed_plugins


def dev_list_plugins(
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Lists all installed CLI plugins.

    This command acts as a wrapper around the shared `handle_list_plugins`
    utility to provide a consistent interface for developers.

    Args:
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
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

    validate_common_flags(fmt, command, quiet)
    effective, _, _ = resolve_command_config(
        command=command,
        quiet=quiet,
        verbose=verbose,
        log_level=log_level,
        fmt=fmt,
        pretty=pretty,
    )
    plugins = list_installed_plugins()

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        payload: dict[str, object] = {"plugins": plugins}
        if include_runtime:
            payload["python"] = platform.python_version()
            payload["platform"] = platform.platform()
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=effective.quiet,
        verbose=effective.verbose_level > 0,
        fmt=effective.fmt,
        pretty=effective.pretty,
        log_level=effective.log_level,
    )
