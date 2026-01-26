# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the root callback for the `bijux memory` command group.

This module defines the default action for the `bijux memory` command. When
invoked without a subcommand, it provides a summary of the transient,
in-memory data store, including the number of keys currently set.

Output Contract:
    * Success: `{"status": "ok", "count": int|None, "message": str}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred (e.g., service unavailable).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

import contextlib
import platform
import sys

import typer

from bijux_cli.cli.color import resolve_click_color
from bijux_cli.cli.commands.memory.resolve import resolve_memory_service
from bijux_cli.cli.core.command import (
    ascii_safe,
    contains_non_ascii_env,
    normalize_format,
    raise_exit_intent,
    record_history,
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
from bijux_cli.core.enums import ErrorType, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.precedence import current_execution_policy


def _build_payload(include_runtime: bool, keys_count: int | None) -> dict[str, object]:
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
        return {
            "status": payload["status"],
            "count": payload["count"],
            "message": payload["message"],
            "python": ascii_safe(platform.python_version(), "python_version"),
            "platform": ascii_safe(platform.platform(), "platform"),
        }
    return payload


def _run_one_shot_mode(
    *,
    command: str,
    fmt: OutputFormat,
    output_format: OutputFormat,
    quiet: bool,
    log_level: LogLevel,
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
        log_level (LogLevel): Logging level for diagnostics.
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
        raise_exit_intent(
            "Non-ASCII characters in environment variables",
            code=3,
            failure="ascii_env",
            error_type=ErrorType.ASCII,
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
        )
    try:
        payload = _build_payload(include_runtime, keys_count)
    except ValueError as exc:
        raise_exit_intent(
            str(exc),
            code=3,
            failure="ascii",
            error_type=ErrorType.ASCII,
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
        )

    emit_output = not quiet
    if not emit_output:
        from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError

        raise ExitIntentError(
            ExitIntent(
                code=ExitCode.SUCCESS,
                stream=None,
                payload=None,
                fmt=output_format,
                pretty=effective_pretty,
                show_traceback=False,
            )
        )
    record_history(command, 0)
    from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError

    raise ExitIntentError(
        ExitIntent(
            code=ExitCode.SUCCESS,
            stream="stdout",
            payload=payload,
            fmt=output_format,
            pretty=effective_pretty,
            show_traceback=False,
        )
    )


def memory_summary(
    ctx: typer.Context,
    quiet: bool,
    fmt: str,
    pretty: bool,
    log_level: str,
) -> None:
    """Handles the logic for the default `bijux memory` command action.

    This function is called by the main Typer callback when no subcommand is
    specified. It resolves the memory service, gets the key count, and then
    executes the one-shot summary.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        log_level (str): The requested logging level.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload upon completion or error.
    """
    command = "memory"
    policy = current_execution_policy()
    quiet = policy.quiet
    include_runtime = policy.include_runtime
    log_level_value = policy.log_level
    include_runtime = policy.include_runtime
    effective_pretty = policy.pretty
    fmt_lower = normalize_format(fmt) or OutputFormat.JSON

    validate_common_flags(
        fmt_lower,
        command,
        quiet,
        include_runtime=include_runtime,
        log_level=log_level_value,
    )

    output_format = fmt_lower

    svc = resolve_memory_service(
        command, fmt_lower, quiet, include_runtime, log_level_value
    )

    keys_count = None
    with contextlib.suppress(Exception):
        keys_count = len(svc.keys())

    _run_one_shot_mode(
        command=command,
        fmt=fmt_lower,
        output_format=output_format,
        quiet=quiet,
        log_level=log_level_value,
        effective_pretty=effective_pretty,
        include_runtime=include_runtime,
        keys_count=keys_count,
    )


def memory(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint for the `bijux memory` command group.

    This function serves as the main callback. It handles `--help` requests and,
    if no subcommand is invoked, delegates to the `memory_summary` function to
    display the default summary view.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        log_level (str): The resolved log level name.

    Returns:
        None:

    Raises:
        typer.Exit: Exits after displaying help text.
    """
    if any(arg in ("-h", "--help") for arg in sys.argv):
        policy = current_execution_policy()
        color = resolve_click_color(quiet=policy.quiet, fmt=None)
        if ctx.invoked_subcommand:
            cmd = getattr(ctx.command, "get_command", None)
            sub_cmd = cmd(ctx, ctx.invoked_subcommand) if callable(cmd) else None
            if sub_cmd and hasattr(sub_cmd, "get_help"):
                typer.echo(sub_cmd.get_help(ctx), color=color)
            else:
                typer.echo(ctx.get_help(), color=color)
        else:
            typer.echo(ctx.get_help(), color=color)
        raise typer.Exit()
    if ctx.invoked_subcommand is None:
        memory_summary(
            ctx=ctx,
            quiet=quiet,
            fmt=fmt,
            pretty=pretty,
            log_level=log_level,
        )
