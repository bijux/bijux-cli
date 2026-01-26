# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `config get` subcommand for the Bijux CLI.

This module contains the logic for retrieving the value of a specific key
from the active configuration store. It provides a structured, machine-readable
response containing the value or an error if the key is not found.

Output Contract:
    * Success: `{"value": str}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred while accessing the configuration.
    * `2`: The specified key was not found in the configuration.
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
from bijux_cli.core.enums import ErrorType
from bijux_cli.core.errors import ConfigError
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.core.precedence import current_execution_policy, resolve_exit_intent
from bijux_cli.services.config.contracts import ConfigProtocol


def get_config(
    ctx: typer.Context,
    key: str = typer.Argument(..., help="Configuration key to look up"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Retrieves the value for a given configuration key.

    This function fetches the value for the specified key from the configuration
    service and uses the `new_run_command` helper to emit it in a structured
    payload. It handles errors, such as the key not being found.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        key (str): The configuration key whose value should be retrieved.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing the error.
    """
    command = "config get"
    effective = current_execution_policy()
    fmt_lower = validate_common_flags(
        fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
        log_level=effective.log_level,
    )
    quiet = effective.quiet
    include_runtime = effective.include_runtime
    pretty = effective.pretty

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        value = config_svc.get(key)
    except ConfigError as exc:
        if str(exc).startswith("Config key not found"):
            intent = resolve_exit_intent(
                message=f"Config key not found: {key}",
                code=2,
                failure="not_found",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                error_type=ErrorType.USER_INPUT,
                log_level=effective.log_level,
                extra={"key": key},
            )
            raise ExitIntentError(intent) from exc
        intent = resolve_exit_intent(
            message=f"Failed to get config: {exc}",
            code=1,
            failure="get_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            error_type=ErrorType.INTERNAL,
            log_level=effective.log_level,
        )
        raise ExitIntentError(intent) from exc

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Builds a payload containing the retrieved configuration value.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            dict[str, object]: A dictionary containing the key's value and
                optional runtime metadata.
        """
        payload: dict[str, object] = {"value": value}
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
