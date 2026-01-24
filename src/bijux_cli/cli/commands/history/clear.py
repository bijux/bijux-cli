# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `history clear` subcommand for the Bijux CLI.

This module contains the logic for permanently erasing all entries from the
command history store. This action is irreversible. A structured confirmation
is emitted upon success.

Output Contract:
    * Success: `{"status": "cleared"}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred, such as the history service being
      unavailable or a failure during the clear operation.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.commands.payloads import HistoryClearPayload
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
from bijux_cli.cli.core.validation import ascii_safe, validate_common_flags
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.precedence import LogPolicy
from bijux_cli.services.history.contracts import HistoryProtocol


def resolve_history_service(
    command: str,
    fmt_lower: OutputFormat,
    quiet: bool,
    include_runtime: bool,
    log_policy: LogPolicy,
) -> HistoryProtocol:
    """Resolves the HistoryProtocol implementation from the DI container.

    Args:
        command (str): The full command name (e.g., "history clear").
        fmt_lower (OutputFormat): The chosen output format.
        quiet (bool): If True, suppresses non-error output.
        include_runtime (bool): If True, includes runtime metadata in errors.
        log_policy (LogPolicy): Logging policy for diagnostics.

    Returns:
        HistoryProtocol: An instance of the history service.

    Raises:
        SystemExit: Exits with a structured error if the service cannot be
            resolved from the container.
    """
    try:
        return DIContainer.current().resolve(HistoryProtocol)
    except Exception as exc:
        emit_error_with_policy(
            f"History service unavailable: {exc}",
            code=1,
            failure="service_unavailable",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_policy=log_policy,
        )


def clear_history(
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    verbose: bool = typer.Option(False, *OPT_VERBOSE, help=HELP_VERBOSE),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Erases all stored command history.

    This command permanently removes all entries from the history store and
    emits a structured payload to confirm the operation.

    Args:
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "history clear"
    validate_common_flags(fmt, command, quiet)
    effective, fmt_lower = resolve_command_config(
        command=command,
        fmt=fmt,
    )
    quiet = effective.quiet
    verbose = effective.verbose_level > 0
    log_policy = effective.log_policy
    pretty = effective.pretty
    include_runtime = effective.include_runtime

    history_svc = resolve_history_service(
        command, fmt_lower, quiet, include_runtime, log_policy
    )

    try:
        history_svc.clear()
    except Exception as exc:
        emit_error_with_policy(
            f"Failed to clear history: {exc}",
            code=1,
            failure="clear_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_policy=log_policy,
        )

    def payload_builder(include_runtime: bool) -> HistoryClearPayload:
        """Builds the payload confirming the history was cleared.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            HistoryClearPayload: The structured payload.
        """
        payload = HistoryClearPayload(status="cleared")
        if include_runtime:
            return HistoryClearPayload(
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
