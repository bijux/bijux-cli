# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `status` command for the Bijux CLI.

This module provides a lightweight "liveness probe" for the CLI, designed for
health checks and monitoring. In its default mode, it performs a quick check
and returns a simple "ok" status. It also supports a continuous "watch" mode
that emits status updates at a regular interval.

Output Contract:
    * Success:          `{"status": "ok"}`
    * Watch Mode Tick:  `{"status": "ok", "ts": float, ...}`
    * Watch Mode Stop:  `{"status": "watch-stopped", ...}`
    * Error:            `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: Internal or fatal error during execution.
    * `2`: Invalid argument (e.g., bad watch interval or format).
    * `3`: ASCII encoding error.
"""

from __future__ import annotations

from dataclasses import replace
import platform
import signal
import threading
import time
from types import FrameType

import typer

from bijux_cli.cli.commands.payloads import StatusPayload
from bijux_cli.cli.core.command import (
    emit_error_with_policy,
    new_run_command,
    normalize_payload,
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
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import LogLevel, OutputFormat
from bijux_cli.core.precedence import LogPolicy
from bijux_cli.core.runtime import AsyncTyper
from bijux_cli.infra.contracts import Emitter
from bijux_cli.services.contracts import TelemetryProtocol

typer.core.rich = None  # type: ignore[attr-defined,assignment]

status_app = AsyncTyper(
    name="status",
    help="Show the CLI Status (Lean probe).",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)


def _build_payload(include_runtime: bool) -> StatusPayload:
    """Constructs the status payload.

    Args:
        include_runtime (bool): If True, includes Python version and platform
            information in the payload.

    Returns:
        Mapping[str, object]: A dictionary containing the status and optional
            runtime details.
    """
    payload = StatusPayload(status="ok")
    if include_runtime:
        return replace(
            payload,
            python=ascii_safe(platform.python_version(), "python_version"),
            platform=ascii_safe(platform.platform(), "platform"),
        )
    return payload


def _run_watch_mode(
    *,
    command: str,
    watch_interval: float,
    fmt: OutputFormat,
    quiet: bool,
    effective_pretty: bool,
    include_runtime: bool,
    log_policy: LogPolicy,
    telemetry: TelemetryProtocol,
    emitter: Emitter,
) -> None:
    """Emits CLI status in a continuous watch mode.

    This function enters a loop, emitting a JSON-formatted status payload at
    the specified interval. It handles graceful shutdown on SIGINT (Ctrl+C).

    Args:
        command (str): The command name for telemetry and error contracts.
        watch_interval (float): The polling interval in seconds.
        fmt (str): The output format, which must be "json" for streaming.
        quiet (bool): If True, suppresses all output except errors.
        effective_pretty (bool): If True, pretty-prints the output.
        include_runtime (bool): If True, includes Python and platform fields.
        log_policy (LogPolicy): Logging behavior for debug output.
        telemetry (TelemetryProtocol): The telemetry sink for reporting events.
        emitter (Emitter): The output emitter instance.

    Returns:
        None:

    Raises:
        SystemExit: On an invalid format or an unrecoverable error during
            the watch loop.
    """
    format_value = fmt
    if format_value is not OutputFormat.JSON:
        emit_error_with_policy(
            "Only JSON output is supported in watch mode.",
            code=2,
            failure="watch_fmt",
            command=command,
            fmt=format_value,
            quiet=quiet,
            include_runtime=include_runtime,
            log_policy=log_policy,
        )

    stop = False

    def _sigint_handler(_sig: int, _frame: FrameType | None) -> None:
        """Handles SIGINT to allow for a graceful shutdown of the watch loop.

        Args:
            _sig (int): The signal number (unused).
            _frame (FrameType | None): The current stack frame (unused).
        """
        nonlocal stop
        stop = True

    old_handler = None
    if threading.current_thread() is threading.main_thread():
        try:
            old_handler = signal.signal(signal.SIGINT, _sigint_handler)
        except ValueError:
            old_handler = None
    try:
        while not stop:
            try:
                payload = replace(_build_payload(include_runtime), ts=time.time())
                if log_policy.show_internal and not quiet:
                    emitter.emit(
                        normalize_payload(payload),
                        fmt=OutputFormat.JSON,
                        pretty=effective_pretty,
                        level=LogLevel.DEBUG,
                        message=f"Debug: Emitting payload at ts={payload.ts}",
                        output=None,
                        emit_output=False,
                        emit_diagnostics=True,
                    )
                if not quiet:
                    emitter.emit(
                        normalize_payload(payload),
                        fmt=OutputFormat.JSON,
                        pretty=effective_pretty,
                        level=LogLevel.INFO,
                        message="Status update",
                        output=None,
                        emit_output=not quiet,
                        emit_diagnostics=log_policy.show_internal,
                    )
                telemetry.event(
                    "COMMAND_SUCCESS",
                    {"command": command, "format": fmt.value, "mode": "watch"},
                )
                time.sleep(watch_interval)
            except ValueError as exc:
                emit_error_with_policy(
                    str(exc),
                    code=3,
                    failure="ascii",
                    command=command,
                    fmt=fmt,
                    quiet=quiet,
                    include_runtime=include_runtime,
                    log_policy=log_policy,
                )
            except Exception as exc:
                emit_error_with_policy(
                    f"Watch mode failed: {exc}",
                    code=1,
                    failure="emit",
                    command=command,
                    fmt=fmt,
                    quiet=quiet,
                    include_runtime=include_runtime,
                    log_policy=log_policy,
                )
    finally:
        if old_handler is not None:
            signal.signal(signal.SIGINT, old_handler)
        try:
            stop_payload = replace(
                _build_payload(include_runtime), status="watch-stopped"
            )
            if log_policy.show_internal and not quiet:
                emitter.emit(
                    normalize_payload(stop_payload),
                    fmt=OutputFormat.JSON,
                    pretty=effective_pretty,
                    level=LogLevel.DEBUG,
                    message="Debug: Emitting watch-stopped payload",
                    output=None,
                    emit_output=False,
                    emit_diagnostics=True,
                )
            if not quiet:
                emitter.emit(
                    normalize_payload(stop_payload),
                    fmt=OutputFormat.JSON,
                    pretty=effective_pretty,
                    level=LogLevel.INFO,
                    message="Status watch stopped",
                    output=None,
                    emit_output=not quiet,
                    emit_diagnostics=log_policy.show_internal,
                )
            telemetry.event(
                "COMMAND_STOPPED",
                {"command": command, "format": fmt.value, "mode": "watch"},
            )
        except (ValueError, Exception):
            _ = None


@status_app.callback(invoke_without_command=True)
def status(
    ctx: typer.Context,
    watch: float | None = typer.Option(None, "--watch", help="Poll every N seconds"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint and logic for the `bijux status` command.

    This function orchestrates the status check. It validates flags and then
    dispatches to either the single-run logic or the continuous watch mode
    based on the presence of the `--watch` flag.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        watch (float | None): If provided, enters watch mode, polling at this
            interval in seconds. Must be a positive number.
        quiet (bool): If True, suppresses all output except for errors.
            output payload.
        fmt (str): The output format, either "json" or "yaml". Watch mode only
            supports "json".
        pretty (bool): If True, pretty-prints the output for human readability.
        log_level (str): The resolved log level name.

    Returns:
        None:

    Raises:
        SystemExit: Exits with a contract-compliant status code and payload
            upon any error, such as an invalid watch interval.
    """
    if ctx.invoked_subcommand:
        return

    emitter = DIContainer.current().resolve(Emitter)
    telemetry = DIContainer.current().resolve(TelemetryProtocol)
    command = "status"

    effective, fmt_lower = resolve_command_config(
        command=command,
        fmt=fmt,
    )
    quiet = effective.quiet
    log_policy = effective.log_policy
    pretty = effective.pretty
    validate_common_flags(
        fmt, command, quiet, include_runtime=effective.include_runtime
    )

    if watch is not None:
        try:
            interval = float(watch)
            if interval <= 0:
                raise ValueError
        except (ValueError, TypeError):
            emit_error_with_policy(
                "Invalid watch interval: must be > 0",
                code=2,
                failure="interval",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=effective.include_runtime,
                log_policy=log_policy,
            )

        _run_watch_mode(
            command=command,
            watch_interval=interval,
            fmt=fmt_lower,
            quiet=quiet,
            effective_pretty=pretty,
            include_runtime=effective.include_runtime,
            log_policy=effective.log_policy,
            telemetry=telemetry,
            emitter=emitter,
        )
    else:
        new_run_command(
            command_name=command,
            payload_builder=lambda include: _build_payload(include),
            quiet=quiet,
            fmt=fmt_lower,
            pretty=pretty,
            log_level=log_level,
        )
