# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the root callback for the `bijux config` command group.

This module defines the default action for the `bijux config` command. When
invoked without a subcommand (like `get`, `set`, or `unset`), it lists all
key-value pairs currently stored in the active configuration, presenting them
in a structured, machine-readable format.

Output Contract:
    * Success: `{"KEY_1": "VALUE_1", "KEY_2": "VALUE_2", ...}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred while accessing the configuration.
"""

from __future__ import annotations

import platform

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
from bijux_cli.core.di import DIContainer
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.services.config.contracts import ConfigProtocol


def config(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint for the `bijux config` command group.

    This function serves as the default action when `bijux config` is run
    without a subcommand. It retrieves and displays all key-value pairs from
    the current configuration. If a subcommand (`get`, `set`, etc.) is
    invoked, this function yields control to it.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:
    """
    if ctx.invoked_subcommand:
        return

    command = "config"
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

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Builds the payload containing all configuration values.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            ConfigDumpPayload: A payload containing configuration entries
                and optional runtime metadata.
        """
        data = config_svc.all()
        payload: dict[str, object] = dict(data)
        if include_runtime:
            payload.update(
                {
                    "python": ascii_safe(platform.python_version(), "python_version"),
                    "platform": ascii_safe(platform.platform(), "platform"),
                }
            )
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
