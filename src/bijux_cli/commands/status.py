# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `status` command for the Bijux CLI.

This module provides a lightweight "liveness probe" for the CLI, designed for
health checks and monitoring. In its default mode, it performs a quick check
and returns a simple "ok" status. It also supports a continuous "watch" mode
that emits status updates at a regular interval.

Output Contract:
    * Success:          `{"status": "ok"}`
    * Verbose:          Adds `{"python": str, "platform": str}` to the payload.
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

from collections.abc import Mapping
import platform
import signal
import sys
import time
from types import FrameType

import typer

from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.contracts import EmitterProtocol, TelemetryProtocol
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import OutputFormat

typer.core.rich = None  # type: ignore[attr-defined,assignment]

status_app = typer.Typer(  # pytype: skip-file
    name="status",
    help="Show the CLI Status (Lean probe).",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)


def _build_payload(include_runtime: bool) -> Mapping[str, object]:
    """Constructs the status payload.

    Args:
        include_runtime (bool): If True, includes Python version and platform
            information in the payload.

    Returns:
        Mapping[str, object]: A dictionary containing the status and optional
            runtime details.
    """
    payload: dict[str, object] = {"status": "ok"}
    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
    return payload


def _run_watch_mode(
    *,
    command: str,
    watch_interval: float,
    fmt: str,
    quiet: bool,
    verbose: bool,
    debug: bool,
    effective_pretty: bool,
    include_runtime: bool,
    telemetry: TelemetryProtocol,
    emitter: EmitterProtocol,
) -> None:
    """Emits CLI status in a continuous watch mode.

    This function enters a loop, emitting a JSON-formatted status payload at
    the specified interval. It handles graceful shutdown on SIGINT (Ctrl+C).

    Args:
        command (str): The command name for telemetry and error contracts.
        watch_interval (float): The polling interval in seconds.
        fmt (str): The output format, which must be "json" for streaming.
        quiet (bool): If True, suppresses all output except errors.
        verbose (bool): If True, includes verbose fields in the payload.
        debug (bool): If True, enables diagnostic output to stderr.
        effective_pretty (bool): If True, pretty-prints the output.
        include_runtime (bool): If True, includes Python and platform fields.
        telemetry (TelemetryProtocol): The telemetry sink for reporting events.
        emitter (EmitterProtocol): The output emitter instance.

    Returns:
        None:

    Raises:
        SystemExit: On an invalid format or an unrecoverable error during
            the watch loop.
    """
    if fmt != "json":
        emit_error_and_exit(
            "Only JSON output is supported in watch mode.",
            code=2,
            failure="watch_fmt",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
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

    old_handler = signal.signal(signal.SIGINT, _sigint_handler)
    try:
        while not stop:
            try:
                payload = dict(_build_payload(include_runtime))
                payload["ts"] = time.time()
                if debug and not quiet:
                    print(
                        f"Debug: Emitting payload at ts={payload['ts']}",
                        file=sys.stderr,
                    )
                if not quiet:
                    emitter.emit(
                        payload,
                        fmt=OutputFormat.JSON,
                        pretty=effective_pretty,
                    )
                telemetry.event(
                    "COMMAND_SUCCESS",
                    {"command": command, "format": fmt, "mode": "watch"},
                )
                time.sleep(watch_interval)
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
            except Exception as exc:
                emit_error_and_exit(
                    f"Watch mode failed: {exc}",
                    code=1,
                    failure="emit",
                    command=command,
                    fmt=fmt,
                    quiet=quiet,
                    include_runtime=include_runtime,
                )
    finally:
        signal.signal(signal.SIGINT, old_handler)
        try:
            stop_payload = dict(_build_payload(include_runtime))
            stop_payload["status"] = "watch-stopped"
            if debug and not quiet:
                print("Debug: Emitting watch-stopped payload", file=sys.stderr)
            if not quiet:
                emitter.emit(
                    stop_payload,
                    fmt=OutputFormat.JSON,
                    pretty=effective_pretty,
                    level="info",
                )
            telemetry.event(
                "COMMAND_STOPPED",
                {"command": command, "format": fmt, "mode": "watch"},
            )
        except (ValueError, Exception):
            _ = None


@status_app.callback(invoke_without_command=True)
def status(
    ctx: typer.Context,
    watch: float | None = typer.Option(None, "--watch", help="Poll every N seconds"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
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
        verbose (bool): If True, includes Python and platform details in the
            output payload.
        fmt (str): The output format, either "json" or "yaml". Watch mode only
            supports "json".
        pretty (bool): If True, pretty-prints the output for human readability.
        debug (bool): If True, enables debug diagnostics, implying `verbose`
            and `pretty`.

    Returns:
        None:

    Raises:
        SystemExit: Exits with a contract-compliant status code and payload
            upon any error, such as an invalid watch interval.
    """
    if ctx.invoked_subcommand:
        return

    emitter = DIContainer.current().resolve(EmitterProtocol)
    telemetry = DIContainer.current().resolve(TelemetryProtocol)
    command = "status"

    fmt_lower = validate_common_flags(fmt, command, quiet)

    if watch is not None:
        try:
            interval = float(watch)
            if interval <= 0:
                raise ValueError
        except (ValueError, TypeError):
            emit_error_and_exit(
                "Invalid watch interval: must be > 0",
                code=2,
                failure="interval",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )

        _run_watch_mode(
            command=command,
            watch_interval=interval,
            fmt=fmt_lower,
            quiet=quiet,
            verbose=verbose,
            debug=debug,
            effective_pretty=pretty,
            include_runtime=verbose,
            telemetry=telemetry,
            emitter=emitter,
        )
    else:
        new_run_command(
            command_name=command,
            payload_builder=lambda include: _build_payload(include),
            quiet=quiet,
            verbose=verbose,
            fmt=fmt_lower,
            pretty=pretty,
            debug=debug,
        )
