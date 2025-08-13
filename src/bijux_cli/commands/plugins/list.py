# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins list` subcommand for the Bijux CLI.

This module provides the primary command for listing all installed CLI plugins.
It performs security checks on the plugins directory and then delegates its
core logic to the shared `handle_list_plugins` utility, which scans the
filesystem and returns a structured list.

Output Contract:
    * Success: `{"plugins": [str, ...]}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An error occurred while accessing the plugins directory (e.g.,
      it is a symlink or inaccessible).
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

import typer

from bijux_cli.commands.plugins.utils import refuse_on_symlink
from bijux_cli.commands.utilities import handle_list_plugins, validate_common_flags
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.services.plugins import get_plugins_dir


def list_plugin(
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Lists all installed CLI plugins.

    This command first performs security checks on the plugins directory, such
    as ensuring it is not a symbolic link. It then delegates to the shared
    `handle_list_plugins` utility to perform the filesystem scan and emit the
    structured output.

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
    command = "plugins list"

    fmt_lower = validate_common_flags(fmt, command, quiet)
    plugins_dir = get_plugins_dir()
    refuse_on_symlink(plugins_dir, command, fmt_lower, quiet, verbose, debug)
    handle_list_plugins(command, quiet, verbose, fmt, pretty, debug)
