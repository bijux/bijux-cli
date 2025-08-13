# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `memory delete` subcommand for the Bijux CLI.

This module contains the logic for removing a specific key and its associated
value from the transient, in-memory data store. It provides a structured,
machine-readable confirmation of the deletion.

Output Contract:
    * Success: `{"status": "deleted", "key": str}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: The key was not found, or another unexpected error occurred.
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


def _build_payload(include_runtime: bool, key: str) -> Mapping[str, object]:
    """Builds the payload for a memory key deletion response.

    Args:
        include_runtime (bool): If True, includes Python and platform info.
        key (str): The memory key that was deleted.

    Returns:
        Mapping[str, object]: A dictionary containing the status, the key that
            was deleted, and optional runtime metadata.
    """
    payload: dict[str, object] = {"status": "deleted", "key": key}
    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
    return payload


def delete_memory(
    key: str = typer.Argument(..., help="Key to delete"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Deletes a key from the transient in-memory store.

    This command validates the key's format and then removes the corresponding
    key-value pair from the memory service.

    Args:
        key (str): The memory key to remove. Must be between 1 and 4096
            printable, non-whitespace characters.
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
    command = "memory delete"

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
        memory_svc.delete(key)
    except KeyError:
        emit_error_and_exit(
            f"Key not found: {key}",
            code=1,
            failure="not_found",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to delete memory key: {exc}",
            code=1,
            failure="delete_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, key),
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
