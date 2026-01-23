# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `config reload` subcommand for the Bijux CLI.

This module contains the logic for manually reloading the application's
configuration from its source file on disk. It discards any in-memory
settings and replaces them with the content of the configuration file,
emitting a structured confirmation upon success.

Output Contract:
    * Success: `{"status": "reloaded"}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `2`: The configuration file could not be read or parsed.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.commands.payloads import ConfigReloadPayload
from bijux_cli.cli.constants import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
    OPT_FORMAT,
    OPT_LOG_LEVEL,
    OPT_PRETTY,
    OPT_QUIET,
    OPT_VERBOSE,
)
from bijux_cli.cli.core.emit import emit_error_and_exit
from bijux_cli.cli.core.output import new_run_command, resolve_command_config
from bijux_cli.cli.core.validation import ascii_safe
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import LogLevel
from bijux_cli.services.config.contracts import ConfigProtocol


def reload_config(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    verbose: bool = typer.Option(False, *OPT_VERBOSE, help=HELP_VERBOSE),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Reloads the configuration from disk and emits a structured result.

    This function forces a refresh of the application's configuration from its
    persistent storage file. It is useful when the configuration has been
    modified externally. A success or error payload is always emitted.

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
            payload, indicating success or detailing the error.
    """
    command = "config reload"
    effective, _, fmt_lower = resolve_command_config(
        command=command,
        quiet=quiet,
        verbose=verbose,
        log_level=log_level,
        fmt=fmt,
        pretty=pretty,
    )
    quiet = effective.quiet
    verbose = effective.verbose_level > 0
    debug = effective.log_level == LogLevel.DEBUG
    pretty = effective.pretty
    include_runtime = effective.include_runtime

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        config_svc.reload()
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to reload config: {exc}",
            code=2,
            failure="reload_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    def payload_builder(include_runtime: bool) -> ConfigReloadPayload:
        """Builds the payload confirming a successful configuration reload.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            dict[str, object]: The structured payload.
        """
        payload = ConfigReloadPayload(status="reloaded")
        if include_runtime:
            return ConfigReloadPayload(
                status=payload.status,
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
