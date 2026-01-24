# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `memory get` subcommand for the Bijux CLI.

This module contains the logic for retrieving the value of a specific key
from the transient, in-memory data store. It provides a structured,
machine-readable response containing the value or an error if the key is
not found.

Output Contract:
    * Success: `{"status": "ok", "key": str, "value": str}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: The key was not found, or another unexpected error occurred.
    * `2`: The provided key was invalid.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.commands.memory.resolve import resolve_memory_service
from bijux_cli.cli.commands.payloads import MemoryItemPayload
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
from bijux_cli.cli.core.validation import ascii_safe, validate_common_flags


def _build_payload(include_runtime: bool, key: str, value: str) -> MemoryItemPayload:
    """Builds the payload for a single memory key-value response.

    Args:
        include_runtime (bool): If True, includes Python and platform info.
        key (str): The memory key that was retrieved.
        value (str): The value associated with the key.

    Returns:
        Mapping[str, object]: A dictionary containing the status, key, value,
            and optional runtime metadata.
    """
    payload = MemoryItemPayload(status="ok", key=key, value=value)
    if include_runtime:
        return MemoryItemPayload(
            status=payload.status,
            key=key,
            value=value,
            python=ascii_safe(platform.python_version(), "python_version"),
            platform=ascii_safe(platform.platform(), "platform"),
        )
    return payload


def get_memory(
    key: str = typer.Argument(..., help="Key to retrieve"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Retrieves a value by key from the transient in-memory store.

    This command validates the key's format and then fetches the corresponding
    value from the memory service.

    Args:
        key (str): The memory key to look up. Must be between 1 and 4096
            printable, non-whitespace characters.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "memory get"
    validate_common_flags(fmt, command, quiet)
    effective, fmt_lower = resolve_command_config(
        command=command,
        fmt=fmt,
    )
    quiet = effective.quiet
    include_runtime = effective.include_runtime
    log_policy = effective.log_policy
    pretty = effective.pretty

    if not (
        1 <= len(key) <= 4096 and all(c.isprintable() and not c.isspace() for c in key)
    ):
        emit_error_with_policy(
            "Invalid key: must be 1-4096 printable non-space characters",
            code=2,
            failure="invalid_key",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            log_policy=log_policy,
        )

    memory_svc = resolve_memory_service(
        command, fmt_lower, quiet, include_runtime, log_policy
    )

    try:
        value = memory_svc.get(key)
    except KeyError:
        emit_error_with_policy(
            f"Key not found: {key}",
            code=1,
            failure="not_found",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            log_policy=log_policy,
        )
    except Exception as exc:
        emit_error_with_policy(
            f"Failed to get memory: {exc}",
            code=1,
            failure="get_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            log_policy=log_policy,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, key, value),
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
