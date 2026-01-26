# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `memory list` subcommand for the Bijux CLI.

This module contains the logic for listing all keys currently held in the
transient, in-memory data store. It retrieves the keys and presents them in a
structured, machine-readable list format.

Output Contract:
    * Success: `{"status": "ok", "keys": list, "count": int}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred (e.g., service unavailable, list failed).
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


def _build_payload(include_runtime: bool, keys: list[str]) -> dict[str, object]:
    """Builds the payload for the memory keys list response.

    Args:
        include_runtime (bool): If True, includes Python and platform info.
        keys (list[str]): The list of keys from the memory store.

    Returns:
        Mapping[str, object]: A dictionary containing the status, a sorted list
            of keys, the key count, and optional runtime metadata.
    """
    payload: dict[str, object] = {"status": "ok", "keys": keys, "count": len(keys)}
    if include_runtime:
        return {
            "status": payload["status"],
            "keys": keys,
            "count": len(keys),
            "python": ascii_safe(platform.python_version(), "python_version"),
            "platform": ascii_safe(platform.platform(), "platform"),
        }
    return payload


def list_memory(
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Lists all keys currently stored in the transient in-memory store.

    This command retrieves all defined keys from the memory service, sorts them,
    and then emits them in a structured payload.

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
    command = "memory list"
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
        keys = sorted(memory_svc.keys())
    except Exception as exc:
        raise_exit_intent(
            f"Failed to list memory keys: {exc}",
            code=1,
            failure="list_failed",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, keys),
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level_value,
    )
