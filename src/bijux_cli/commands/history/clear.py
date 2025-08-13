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

from collections.abc import Mapping
import platform
from typing import Any

import typer

from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.contracts import HistoryProtocol
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.di import DIContainer


def resolve_history_service(
    command: str, fmt_lower: str, quiet: bool, include_runtime: bool, debug: bool
) -> HistoryProtocol:
    """Resolves the HistoryProtocol implementation from the DI container.

    Args:
        command (str): The full command name (e.g., "history clear").
        fmt_lower (str): The chosen output format, lowercased.
        quiet (bool): If True, suppresses non-error output.
        include_runtime (bool): If True, includes runtime metadata in errors.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        HistoryProtocol: An instance of the history service.

    Raises:
        SystemExit: Exits with a structured error if the service cannot be
            resolved from the container.
    """
    try:
        return DIContainer.current().resolve(HistoryProtocol)
    except Exception as exc:
        emit_error_and_exit(
            f"History service unavailable: {exc}",
            code=1,
            failure="service_unavailable",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )


def clear_history(
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Erases all stored command history.

    This command permanently removes all entries from the history store and
    emits a structured payload to confirm the operation.

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
    command = "history clear"
    if debug:
        verbose = True
        pretty = True
    include_runtime = verbose

    fmt_lower = validate_common_flags(
        fmt,
        command,
        quiet,
        include_runtime=include_runtime,
    )

    history_svc = resolve_history_service(
        command, fmt_lower, quiet, include_runtime, debug
    )

    try:
        history_svc.clear()
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to clear history: {exc}",
            code=1,
            failure="clear_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    def payload_builder(include_runtime: bool) -> Mapping[str, Any]:
        """Builds the payload confirming the history was cleared.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            Mapping[str, Any]: The structured payload.
        """
        payload: dict[str, Any] = {"status": "cleared"}
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
