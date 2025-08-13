# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the root callback for the `bijux memory` command group.

This module defines the default action for the `bijux memory` command. When
invoked without a subcommand, it provides a summary of the transient,
in-memory data store, including the number of keys currently set.

Output Contract:
    * Success: `{"status": "ok", "count": int|None, "message": str}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred (e.g., service unavailable).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

from collections.abc import Mapping
import contextlib
import platform
import sys

import typer

from bijux_cli.commands.memory.utils import resolve_memory_service
from bijux_cli.commands.utilities import (
    ascii_safe,
    contains_non_ascii_env,
    emit_and_exit,
    emit_error_and_exit,
    validate_common_flags,
)
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.enums import OutputFormat


def _build_payload(
    include_runtime: bool, keys_count: int | None
) -> Mapping[str, object]:
    """Constructs the payload for the memory summary command.

    Args:
        include_runtime (bool): If True, includes Python and platform info.
        keys_count (int | None): The number of keys in the memory store, or
            None if the count could not be determined.

    Returns:
        Mapping[str, object]: A dictionary containing the status, key count,
            a confirmation message, and optional runtime metadata.
    """
    payload: dict[str, object] = {
        "status": "ok",
        "count": keys_count,
        "message": "Memory command executed",
    }
    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
    return payload


def _run_one_shot_mode(
    *,
    command: str,
    fmt: str,
    output_format: OutputFormat,
    quiet: bool,
    verbose: bool,
    debug: bool,
    effective_pretty: bool,
    include_runtime: bool,
    keys_count: int | None,
) -> None:
    """Orchestrates the execution for a single memory summary request.

    This helper function handles environment validation, payload construction,
    and final emission for the memory summary.

    Args:
        command (str): The command name for telemetry and error context.
        fmt (str): The output format string (e.g., "json").
        output_format (OutputFormat): The output format enum for serialization.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes runtime metadata in the payload.
        debug (bool): If True, enables debug diagnostics.
        effective_pretty (bool): If True, pretty-prints the output.
        include_runtime (bool): If True, includes Python/platform info.
        keys_count (int | None): The number of keys in the memory store.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload upon completion or error.
    """
    if contains_non_ascii_env():
        emit_error_and_exit(
            "Non-ASCII characters in environment variables",
            code=3,
            failure="ascii_env",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
        )

    try:
        payload = _build_payload(include_runtime, keys_count)
    except ValueError as exc:
        emit_error_and_exit(
            str(exc),
            code=3,
            failure="ascii",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
        )

    emit_and_exit(
        payload=payload,
        fmt=output_format,
        effective_pretty=effective_pretty,
        verbose=verbose,
        debug=debug,
        quiet=quiet,
        command=command,
        exit_code=0,
    )


def memory_summary(
    ctx: typer.Context,
    quiet: bool,
    verbose: bool,
    fmt: str,
    pretty: bool,
    debug: bool,
) -> None:
    """Handles the logic for the default `bijux memory` command action.

    This function is called by the main Typer callback when no subcommand is
    specified. It resolves the memory service, gets the key count, and then
    executes the one-shot summary.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload upon completion or error.
    """
    command = "memory"
    include_runtime = verbose or debug

    fmt_lower = validate_common_flags(fmt, command, quiet)

    output_format = OutputFormat.YAML if fmt_lower == "yaml" else OutputFormat.JSON
    effective_pretty = debug or pretty

    svc = resolve_memory_service(command, fmt_lower, quiet, include_runtime, debug)

    keys_count = None
    with contextlib.suppress(Exception):
        keys_count = len(svc.keys())

    _run_one_shot_mode(
        command=command,
        fmt=fmt_lower,
        output_format=output_format,
        quiet=quiet,
        verbose=verbose,
        debug=debug,
        effective_pretty=effective_pretty,
        include_runtime=include_runtime,
        keys_count=keys_count,
    )


def memory(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Defines the entrypoint for the `bijux memory` command group.

    This function serves as the main callback. It handles `--help` requests and,
    if no subcommand is invoked, delegates to the `memory_summary` function to
    display the default summary view.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes runtime metadata in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        typer.Exit: Exits after displaying help text.
    """
    if any(arg in ("-h", "--help") for arg in sys.argv):
        if ctx.invoked_subcommand:
            cmd = getattr(ctx.command, "get_command", None)
            sub_cmd = cmd(ctx, ctx.invoked_subcommand) if callable(cmd) else None
            if sub_cmd and hasattr(sub_cmd, "get_help"):
                typer.echo(
                    sub_cmd.get_help(ctx)  # pyright: ignore[reportAttributeAccessIssue]
                )
            else:
                typer.echo(ctx.get_help())
        else:
            typer.echo(ctx.get_help())
        raise typer.Exit()
    if ctx.invoked_subcommand is None:
        memory_summary(ctx, quiet, verbose, fmt, pretty, debug)
