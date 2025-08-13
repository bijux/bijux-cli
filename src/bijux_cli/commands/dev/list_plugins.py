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

import typer

from bijux_cli.commands.utilities import handle_list_plugins, validate_common_flags
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)


def dev_list_plugins(
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Lists all installed CLI plugins.

    This command acts as a wrapper around the shared `handle_list_plugins`
    utility to provide a consistent interface for developers.

    Args:
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
    command = "dev list-plugins"

    validate_common_flags(fmt, command, quiet)

    handle_list_plugins(command, quiet, verbose, fmt, pretty, debug)
