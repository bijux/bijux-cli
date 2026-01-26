# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `memory set` subcommand for the Bijux CLI.

This module contains the logic for storing a key-value pair in a transient,
in-memory data store. The data persists only for the lifetime of the
application's parent process. A structured confirmation is emitted upon success.

Output Contract:
    * Success: `{"status": "updated", "key": str, "value": str}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred (e.g., service unavailable, set failed).
    * `2`: The provided key was invalid.
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


def _build_payload(include_runtime: bool, key: str, value: str) -> dict[str, object]:
    """Constructs the payload confirming a key-value pair was set.

    Args:
        include_runtime (bool): If True, includes Python and platform info.
        key (str): The key that was updated.
        value (str): The new value assigned to the key.

    Returns:
        Mapping[str, object]: A dictionary containing the status, key, value,
            and optional runtime metadata.
    """
    payload: dict[str, object] = {"status": "updated", "key": key, "value": value}
    if include_runtime:
        return {
            "status": payload["status"],
            "key": key,
            "value": value,
            "python": ascii_safe(platform.python_version(), "python_version"),
            "platform": ascii_safe(platform.platform(), "platform"),
        }
    return payload


def set_memory(
    key: str = typer.Argument(..., help="Key to set"),
    value: str = typer.Argument(..., help="Value to set"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Sets a key-value pair in the transient in-memory store.

    This command validates the key's format and then stores the key-value
    pair using the memory service.

    Args:
        key (str): The memory key to set. Must be between 1 and 4096 printable,
            non-whitespace characters.
        value (str): The value to associate with the key.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "memory set"
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

    if not (
        1 <= len(key) <= 4096 and all(c.isprintable() and not c.isspace() for c in key)
    ):
        raise_exit_intent(
            "Invalid key: must be 1-4096 printable non-space characters",
            code=2,
            failure="invalid_key",
            error_type=ErrorType.USER_INPUT,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    memory_svc = resolve_memory_service(
        command, fmt_lower, quiet, include_runtime, log_level_value
    )

    try:
        memory_svc.set(key, value)
    except Exception as exc:
        raise_exit_intent(
            f"Failed to set memory: {exc}",
            code=1,
            failure="set_failed",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, key, value),
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
