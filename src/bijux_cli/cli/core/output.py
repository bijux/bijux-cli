# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""CLI output helpers for resolving config and emitting payloads."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any, NoReturn

from bijux_cli.cli.core.emit import emit_and_exit, emit_error_and_exit
from bijux_cli.cli.core.validation import validate_common_flags
from bijux_cli.core.enums import ErrorType, ExitCode, OutputFormat
from bijux_cli.core.exit_policy import (
    ExitIntent,
    ExitIntentError,
    resolve_exit_behavior,
)
from bijux_cli.core.precedence import ExecutionPolicy, LogPolicy


def current_execution_policy() -> ExecutionPolicy:
    """Return the execution policy resolved during bootstrap."""
    from bijux_cli.core.di import DIContainer

    policy = DIContainer.current().resolve(ExecutionPolicy)
    if not isinstance(policy, ExecutionPolicy):
        raise TypeError("ExecutionPolicy not available")
    return policy


def resolve_command_config(
    *,
    command: str,
    fmt: str,
) -> tuple[ExecutionPolicy, OutputFormat]:
    """Resolve the shared policy for a command invocation."""
    effective = current_execution_policy()
    format_source = fmt if isinstance(fmt, str) else effective.output_format.value
    output_format = validate_common_flags(
        format_source,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
    )
    return effective, output_format


def new_run_command(
    command_name: str,
    payload_builder: Callable[[bool], object],
    quiet: bool,
    verbose: bool,
    fmt: OutputFormat,
    pretty: bool,
    log_level: str,
    exit_code: int = 0,
) -> None:
    """Build and emit a command payload using resolved config."""
    from bijux_cli.core.di import DIContainer
    from bijux_cli.infra.contracts import Emitter
    from bijux_cli.services.contracts import TelemetryProtocol

    _ = (quiet, verbose, fmt, pretty, log_level)
    DIContainer.current().resolve(Emitter)
    DIContainer.current().resolve(TelemetryProtocol)

    resolved = current_execution_policy()
    include_runtime = resolved.include_runtime

    format_source = fmt
    output_format = validate_common_flags(
        format_source,
        command_name,
        resolved.quiet,
        include_runtime=include_runtime,
    )
    effective_pretty = resolved.pretty
    try:
        payload = payload_builder(include_runtime)
    except ValueError as exc:
        emit_error_with_policy(
            str(exc),
            code=2,
            failure="ascii",
            command=command_name,
            fmt=output_format,
            quiet=resolved.quiet,
            include_runtime=include_runtime,
            error_type=ErrorType.ASCII,
            log_policy=resolved.log_policy,
        )
    else:
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
        emit_and_exit(
            payload=payload,
            fmt=output_format,
            effective_pretty=effective_pretty,
            verbose=resolved.verbose,
            command=command_name,
            exit_code=exit_code,
        )


def emit_error_with_policy(
    message: str,
    code: int,
    failure: str,
    *,
    command: str | None = None,
    fmt: OutputFormat | None = None,
    quiet: bool,
    include_runtime: bool = False,
    extra: dict[str, Any] | None = None,
    error_type: ErrorType | None = None,
    log_policy: LogPolicy,
) -> NoReturn:
    """Resolve error behavior and emit a structured error intent."""
    behavior = resolve_exit_behavior(
        error_type or ErrorType.INTERNAL,
        quiet=quiet,
        fmt=fmt or OutputFormat.JSON,
        log_policy=log_policy,
    )
    emit_error_and_exit(
        message,
        code=int(behavior.code),
        failure=failure,
        command=command,
        fmt=fmt,
        include_runtime=include_runtime,
        extra=extra,
        stream=behavior.stream,
        show_traceback=behavior.show_traceback,
    )


__all__ = [
    "current_execution_policy",
    "emit_error_with_policy",
    "resolve_command_config",
    "new_run_command",
]
