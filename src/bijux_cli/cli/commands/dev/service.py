# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the root callback for the `bijux dev` command group.

This module defines the default action for the `bijux dev` command. This command
group is intended for developers of the CLI. When invoked without a subcommand,
it provides a simple status confirmation.

Output Contract:
    * Success: `{"status": "ok"}`
    * With Env Var: Adds `{"mode": str}` if the dev-mode env var is set.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An internal or unexpected error occurred.
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

import os
import platform

import typer

from bijux_cli.cli.commands.payloads import DevStatusPayload
from bijux_cli.cli.core.command import (
    ascii_safe,
    new_run_command,
    normalize_format,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
    ENV_DEV_MODE,
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
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.precedence import current_execution_policy


def dev(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint for the `bijux dev` command group.

    This function serves as the default action when `bijux dev` is run
    without a subcommand. It emits a simple status payload. If a subcommand
    is invoked, this function yields control to it.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload upon completion or error.
    """
    if ctx.invoked_subcommand:
        return

    command = "dev"
    policy = current_execution_policy()
    quiet = policy.quiet
    effective_include_runtime = policy.include_runtime
    effective_pretty = policy.pretty
    fmt_lower = normalize_format(fmt) or OutputFormat.JSON

    validate_common_flags(
        fmt_lower,
        command,
        quiet,
        include_runtime=effective_include_runtime,
    )

    mode = os.environ.get(ENV_DEV_MODE)

    def payload_builder(_: bool) -> DevStatusPayload:
        """Builds the payload for the dev status command.

        The payload indicates an "ok" status and includes optional mode and
        runtime information based on the parent function's scope.

        Args:
            _ (bool): An unused parameter to match the expected signature of
                the `payload_builder` in `new_run_command`.

        Returns:
            DevStatusPayload: The structured payload.
        """
        payload = DevStatusPayload(status="ok")
        if mode:
            payload = DevStatusPayload(
                status=payload.status,
                mode=mode,
                python=payload.python,
                platform=payload.platform,
            )
        if effective_include_runtime:
            payload = DevStatusPayload(
                status=payload.status,
                mode=payload.mode,
                python=ascii_safe(platform.python_version(), "python_version"),
                platform=ascii_safe(platform.platform(), "platform"),
            )
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=quiet,
        fmt=fmt_lower,
        pretty=effective_pretty,
        log_level=log_level,
    )
