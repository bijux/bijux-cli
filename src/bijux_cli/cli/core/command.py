# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Command helpers for policy-aware execution and exit intents."""

from __future__ import annotations

import sys
from typing import Any, NoReturn

from bijux_cli.cli.core.validation import validate_common_flags
from bijux_cli.core.enums import ErrorType, ExitCode, OutputFormat
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError
from bijux_cli.core.precedence import current_execution_policy, resolve_exit_intent


def record_history(command: str, exit_code: int) -> None:
    """Record a history entry, ignoring failures."""
    if command == "history":
        return
    try:
        from bijux_cli.core.di import DIContainer
        from bijux_cli.services.history.contracts import HistoryProtocol

        hist = DIContainer.current().resolve(HistoryProtocol)
        hist.add(
            command=command,
            params=[],
            success=(exit_code == 0),
            return_code=exit_code,
            duration_ms=0.0,
        )
    except PermissionError as exc:
        print(f"Permission denied writing history: {exc}", file=sys.stderr)
    except OSError as exc:
        import errno as _errno

        if exc.errno in (_errno.EACCES, _errno.EPERM):
            print(f"Permission denied writing history: {exc}", file=sys.stderr)
        elif exc.errno in (_errno.ENOSPC, _errno.EDQUOT):
            print(
                f"No space left on device while writing history: {exc}",
                file=sys.stderr,
            )
        else:
            print(f"Error writing history: {exc}", file=sys.stderr)
    except Exception as exc:
        print(f"Error writing history: {exc}", file=sys.stderr)


def new_run_command(
    command_name: str,
    payload_builder: Any,
    quiet: bool,
    fmt: OutputFormat,
    pretty: bool,
    log_level: str,
    exit_code: int = 0,
) -> NoReturn:
    """Build a payload and raise an ExitIntentError with resolved behavior."""
    from bijux_cli.core.di import DIContainer
    from bijux_cli.infra.contracts import Emitter
    from bijux_cli.services.contracts import TelemetryProtocol

    _ = (quiet, fmt, pretty, log_level)
    DIContainer.current().resolve(Emitter)
    DIContainer.current().resolve(TelemetryProtocol)

    resolved = current_execution_policy()
    include_runtime = resolved.include_runtime
    output_format = validate_common_flags(
        fmt,
        command_name,
        resolved.quiet,
        include_runtime=include_runtime,
        log_level=resolved.log_level,
    )
    effective_pretty = resolved.pretty
    try:
        payload = payload_builder(include_runtime)
    except ValueError as exc:
        intent = resolve_exit_intent(
            message=str(exc),
            code=2,
            failure="ascii",
            command=command_name,
            fmt=output_format,
            quiet=resolved.quiet,
            include_runtime=include_runtime,
            error_type=ErrorType.ASCII,
            log_level=resolved.log_level,
        )
        raise ExitIntentError(intent) from exc

    record_history(command_name, exit_code)

    if resolved.quiet:
        intent = ExitIntent(
            code=ExitCode(exit_code),
            stream=None,
            payload=None,
            fmt=output_format,
            pretty=effective_pretty,
            show_traceback=False,
        )
        raise ExitIntentError(intent)

    intent = ExitIntent(
        code=ExitCode(exit_code),
        stream="stdout",
        payload=payload,
        fmt=output_format,
        pretty=effective_pretty,
        show_traceback=False,
    )
    raise ExitIntentError(intent)


def raise_exit_intent(*args: Any, **kwargs: Any) -> NoReturn:
    """Raise an ExitIntentError from resolved error intent."""
    if args:
        if len(args) != 1:
            raise TypeError("raise_exit_intent accepts at most one positional arg")
        kwargs["message"] = args[0]
    raise ExitIntentError(resolve_exit_intent(**kwargs))


__all__ = [
    "new_run_command",
    "record_history",
    "raise_exit_intent",
]
