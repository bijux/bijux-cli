# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the root callback for the `bijux config` command group.

This module defines the default action for the `bijux config` command. When
invoked without a subcommand (like `get`, `set`, or `unset`), it lists all
key-value pairs currently stored in the active configuration, presenting them
in a structured, machine-readable format.

Output Contract:
    * Success: `{"KEY_1": "VALUE_1", "KEY_2": "VALUE_2", ...}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred while accessing the configuration.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.commands.utilities import ascii_safe, new_run_command, parse_global_flags
from bijux_cli.contracts import ConfigProtocol
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.di import DIContainer


def config(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Defines the entrypoint for the `bijux config` command group.

    This function serves as the default action when `bijux config` is run
    without a subcommand. It retrieves and displays all key-value pairs from
    the current configuration. If a subcommand (`get`, `set`, etc.) is
    invoked, this function yields control to it.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:
    """
    if ctx.invoked_subcommand:
        return

    flags = parse_global_flags()

    quiet = flags["quiet"]
    verbose = flags["verbose"]
    fmt = flags["format"]
    pretty = flags["pretty"]
    debug = flags["debug"]

    fmt_lower = fmt.lower()

    command = "config"

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Builds the payload containing all configuration values.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            dict[str, object]: A dictionary of all configuration key-value
                pairs and optional runtime metadata.
        """
        data = config_svc.all()
        payload: dict[str, object] = dict(data)
        if include_runtime:
            payload["python"] = ascii_safe(platform.python_version(), "python_version")
            payload["platform"] = ascii_safe(platform.platform(), "platform")
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
