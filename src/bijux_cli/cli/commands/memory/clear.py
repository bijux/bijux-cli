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

import platform

import typer

from bijux_cli.cli.commands.memory.resolve import resolve_memory_service
from bijux_cli.cli.commands.payloads import MemoryClearPayload
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


def _build_payload(include_runtime: bool) -> MemoryClearPayload:
    """Builds the payload confirming that the in-memory store was cleared.

    Args:
        include_runtime (bool): If True, includes Python and platform info.

    Returns:
        Mapping[str, object]: A dictionary containing the status, a count of 0,
            and optional runtime metadata.
    """
    payload = MemoryClearPayload(status="cleared", count=0)
    if include_runtime:
        return MemoryClearPayload(
            status=payload.status,
            count=payload.count,
            python=ascii_safe(platform.python_version(), "python_version"),
            platform=ascii_safe(platform.platform(), "platform"),
        )
    return payload


def clear_memory(
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    verbose: bool = typer.Option(False, *OPT_VERBOSE, help=HELP_VERBOSE),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Removes all key-value pairs from the transient in-memory store.

    This command erases all entries from the memory service and emits a
    structured payload to confirm the operation.

    Args:
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "memory clear"
    validate_common_flags(fmt, command, quiet)

    effective, fmt_lower = resolve_command_config(
        command=command,
        fmt=fmt,
    )
    quiet = effective.quiet
    verbose = effective.verbose_level > 0
    log_policy = effective.log_policy
    pretty = effective.pretty

    memory_svc = resolve_memory_service(command, fmt_lower, quiet, verbose, log_policy)

    try:
        memory_svc.clear()
    except Exception as exc:
        emit_error_with_policy(
            f"Failed to clear memory: {exc}",
            code=1,
            failure="clear_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            log_policy=log_policy,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include),
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
