# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the root callback for the `bijux dev` command group.

This module defines the default action for the `bijux dev` command. This command
group is intended for developers of the CLI. When invoked without a subcommand,
it provides a simple status confirmation.

Output Contract:
    * Success: `{"status": "ok"}`
    * With Env Var: Adds `{"mode": str}` if `BIJUXCLI_DEV_MODE` is set.
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An internal or unexpected error occurred.
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

from collections.abc import Mapping
import os
import platform
from typing import Any

import typer

from bijux_cli.cli.commands.utilities import (
    ascii_safe,
    effective_defaults,
    new_run_command,
    normalize_format,
    validate_common_flags,
)
from bijux_cli.cli.constants import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.precedence import resolve_effective_config


def dev(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint for the `bijux dev` command group.

    This function serves as the default action when `bijux dev` is run
    without a subcommand. It emits a simple status payload. If a subcommand
    is invoked, this function yields control to it.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload upon completion or error.
    """
    if ctx.invoked_subcommand:
        return

    command = "dev"
    resolved = resolve_effective_config(
        cli={
            "quiet": quiet,
            "verbose": verbose,
            "log_level": log_level,
            "pretty": pretty,
            "format": fmt,
        },
        env={},
        file={},
        defaults=effective_defaults(),
    )
    quiet = resolved.quiet
    verbose = resolved.verbose_level > 0
    effective_include_runtime = resolved.include_runtime
    effective_pretty = resolved.pretty
    fmt_lower = normalize_format(resolved.fmt) or "json"

    validate_common_flags(
        resolved.fmt,
        command,
        quiet,
        include_runtime=effective_include_runtime,
    )

    mode = os.environ.get("BIJUXCLI_DEV_MODE")

    def payload_builder(_: bool) -> Mapping[str, Any]:
        """Builds the payload for the dev status command.

        The payload indicates an "ok" status and includes optional mode and
        runtime information based on the parent function's scope.

        Args:
            _ (bool): An unused parameter to match the expected signature of
                the `payload_builder` in `new_run_command`.

        Returns:
            Mapping[str, Any]: The structured payload.
        """
        payload: dict[str, Any] = {"status": "ok"}
        if mode:
            payload["mode"] = mode
        if effective_include_runtime:
            payload["python"] = ascii_safe(platform.python_version(), "python_version")
            payload["platform"] = ascii_safe(platform.platform(), "platform")
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=effective_pretty,
        log_level=log_level,
    )
