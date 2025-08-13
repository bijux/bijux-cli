# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `memory set` subcommand for the Bijux CLI.

This module contains the logic for storing a key-value pair in a transient,
in-memory data store. The data persists only for the lifetime of the
application's parent process. A structured confirmation is emitted upon success.

Output Contract:
    * Success: `{"status": "updated", "key": str, "value": str}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred (e.g., service unavailable, set failed).
    * `2`: The provided key was invalid.
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


def _build_payload(include_runtime: bool, key: str, value: str) -> Mapping[str, object]:
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
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
    return payload


def set_memory(
    key: str = typer.Argument(..., help="Key to set"),
    value: str = typer.Argument(..., help="Value to set"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Sets a key-value pair in the transient in-memory store.

    This command validates the key's format and then stores the key-value
    pair using the memory service.

    Args:
        key (str): The memory key to set. Must be between 1 and 4096 printable,
            non-whitespace characters.
        value (str): The value to associate with the key.
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
    command = "memory set"

    fmt_lower = validate_common_flags(fmt, command, quiet)

    if not (
        1 <= len(key) <= 4096 and all(c.isprintable() and not c.isspace() for c in key)
    ):
        emit_error_and_exit(
            "Invalid key: must be 1-4096 printable non-space characters",
            code=2,
            failure="invalid_key",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    memory_svc = resolve_memory_service(command, fmt_lower, quiet, verbose, debug)

    try:
        memory_svc.set(key, value)
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to set memory: {exc}",
            code=1,
            failure="set_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, key, value),
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
