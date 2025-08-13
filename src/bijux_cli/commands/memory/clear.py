# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `memory clear` subcommand for the Bijux CLI.

This module contains the logic for permanently erasing all entries from the
transient, in-memory data store. This action is irreversible for the current
process. A structured confirmation is emitted upon success.

Output Contract:
    * Success: `{"status": "cleared", "count": 0}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred (e.g., service unavailable, clear failed).
"""

from __future__ import annotations

from collections.abc import Mapping
import platform

import typer

from bijux_cli.commands.memory.utils import resolve_memory_service
from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)


def _build_payload(include_runtime: bool) -> Mapping[str, object]:
    """Builds the payload confirming that the in-memory store was cleared.

    Args:
        include_runtime (bool): If True, includes Python and platform info.

    Returns:
        Mapping[str, object]: A dictionary containing the status, a count of 0,
            and optional runtime metadata.
    """
    payload: dict[str, object] = {"status": "cleared", "count": 0}
    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
    return payload


def clear_memory(
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Removes all key-value pairs from the transient in-memory store.

    This command erases all entries from the memory service and emits a
    structured payload to confirm the operation.

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
    command = "memory clear"

    fmt_lower = validate_common_flags(fmt, command, quiet)

    memory_svc = resolve_memory_service(command, fmt_lower, quiet, verbose, debug)

    try:
        memory_svc.clear()
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to clear memory: {exc}",
            code=1,
            failure="clear_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include),
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
