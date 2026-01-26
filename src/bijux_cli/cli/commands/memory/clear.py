# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `memory clear` subcommand for the Bijux CLI.

This module contains the logic for permanently erasing all entries from the
transient, in-memory data store. This action is irreversible for the current
process. A structured confirmation is emitted upon success.

Output Contract:
    * Success: `{"status": "cleared", "count": 0}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred (e.g., service unavailable, clear failed).
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.commands.memory.resolve import resolve_memory_service
from bijux_cli.cli.core.command import (
    ascii_safe,
    new_run_command,
    raise_exit_intent,
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
from bijux_cli.core.enums import ErrorType
from bijux_cli.core.precedence import current_execution_policy


def _build_payload(include_runtime: bool) -> dict[str, object]:
    """Builds the payload confirming that the in-memory store was cleared.

    Args:
        include_runtime (bool): If True, includes Python and platform info.

    Returns:
        Mapping[str, object]: A dictionary containing the status, a count of 0,
            and optional runtime metadata.
    """
    payload: dict[str, object] = {"status": "cleared", "count": 0}
    if include_runtime:
        return {
            "status": payload["status"],
            "count": payload["count"],
            "python": ascii_safe(platform.python_version(), "python_version"),
            "platform": ascii_safe(platform.platform(), "platform"),
        }
    return payload


def clear_memory(
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Removes all key-value pairs from the transient in-memory store.

    This command erases all entries from the memory service and emits a
    structured payload to confirm the operation.

    Args:
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "memory clear"
    policy = current_execution_policy()
    quiet = policy.quiet
    include_runtime = policy.include_runtime
    pretty = policy.pretty
    log_level_value = policy.log_level
    fmt_lower = validate_common_flags(
        fmt,
        command,
        quiet,
        include_runtime=include_runtime,
        log_level=log_level_value,
    )

    memory_svc = resolve_memory_service(
        command, fmt_lower, quiet, include_runtime, log_level_value
    )

    try:
        memory_svc.clear()
    except Exception as exc:
        raise_exit_intent(
            f"Failed to clear memory: {exc}",
            code=1,
            failure="clear_failed",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include),
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
