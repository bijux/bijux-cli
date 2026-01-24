# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `config reload` subcommand for the Bijux CLI.

This module contains the logic for manually reloading the application's
configuration from its source file on disk. It discards any in-memory
settings and replaces them with the content of the configuration file,
emitting a structured confirmation upon success.

Output Contract:
    * Success: `{"status": "reloaded"}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `2`: The configuration file could not be read or parsed.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.commands.payloads import ConfigReloadPayload
from bijux_cli.cli.core.command import (
    emit_error_with_policy,
    new_run_command,
    resolve_command_config,
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
from bijux_cli.cli.core.validation import ascii_safe
from bijux_cli.core.di import DIContainer
from bijux_cli.services.config.contracts import ConfigProtocol


def reload_config(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
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
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing the error.
    """
    command = "config reload"
    effective, fmt_lower = resolve_command_config(
        command=command,
        fmt=fmt,
    )
    quiet = effective.quiet
    include_runtime = effective.include_runtime
    log_policy = effective.log_policy
    pretty = effective.pretty
    include_runtime = effective.include_runtime

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        config_svc.reload()
    except Exception as exc:
        emit_error_with_policy(
            f"Failed to reload config: {exc}",
            code=2,
            failure="reload_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_policy=log_policy,
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
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
