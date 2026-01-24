# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `config load` subcommand for the Bijux CLI.

This module contains the logic for replacing the application's entire
configuration with the contents of a specified file. It discards any
in-memory settings and loads the new configuration, emitting a structured
confirmation upon success.

Output Contract:
    * Success: `{"status": "loaded", "file": str}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `2`: The specified file could not be found, read, or parsed.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.commands.payloads import ConfigLoadPayload
from bijux_cli.cli.core.constants import (
    OPT_FORMAT,
    OPT_LOG_LEVEL,
    OPT_PRETTY,
    OPT_QUIET,
    OPT_VERBOSE,
)
from bijux_cli.cli.core.help_text import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.core.output import (
    emit_error_with_policy,
    new_run_command,
    resolve_command_config,
)
from bijux_cli.cli.core.validation import ascii_safe
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ErrorType
from bijux_cli.services.config.contracts import ConfigProtocol


def load_config(
    ctx: typer.Context,
    path: str = typer.Argument(..., help="Path to load from"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    verbose: bool = typer.Option(False, *OPT_VERBOSE, help=HELP_VERBOSE),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Loads configuration from a specified file.

    This function replaces the current in-memory configuration with the
    contents of the file at the given path. It provides a structured payload
    to confirm the operation was successful.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        path (str): The path to the configuration file to load.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing the error.
    """
    command = "config load"
    effective, fmt_lower = resolve_command_config(
        command=command,
        fmt=fmt,
    )
    quiet = effective.quiet
    verbose = effective.verbose_level > 0
    log_policy = effective.log_policy
    pretty = effective.pretty
    include_runtime = effective.include_runtime

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        config_svc.load(path)
    except Exception as exc:
        emit_error_with_policy(
            f"Failed to load config: {exc}",
            code=2,
            failure="load_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_policy=log_policy,
            extra={"path": path},
            error_type=ErrorType.USER_INPUT,
        )

    def payload_builder(include_runtime: bool) -> ConfigLoadPayload:
        """Builds the payload confirming a successful configuration load.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            dict[str, object]: The structured payload.
        """
        payload = ConfigLoadPayload(status="loaded", file=path)
        if include_runtime:
            return ConfigLoadPayload(
                status=payload.status,
                file=payload.file,
                python=ascii_safe(platform.python_version(), "python_version"),
                platform=ascii_safe(platform.platform(), "platform"),
            )
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
