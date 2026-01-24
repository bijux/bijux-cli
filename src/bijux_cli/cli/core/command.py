# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Command helpers for policy-aware execution and exit intents."""

from __future__ import annotations

from dataclasses import fields, is_dataclass
from enum import Enum
import sys
import time
from typing import Any, NoReturn

from bijux_cli.cli.core.validation import ascii_safe, validate_common_flags
from bijux_cli.core.enums import ErrorType, ExitCode, OutputFormat
from bijux_cli.core.exit_policy import (
    ExitIntent,
    ExitIntentError,
    resolve_exit_behavior,
)
from bijux_cli.core.precedence import ExecutionPolicy, LogPolicy


def normalize_payload(obj: Any) -> Any:
    """Normalize dataclasses/enums into plain serializable structures."""
    if is_dataclass(obj):
        from bijux_cli.cli.commands.payloads import ConfigDumpPayload

        if isinstance(obj, ConfigDumpPayload):
            merged: dict[str, Any] = dict(obj.entries)
            if obj.python is not None:
                merged["python"] = obj.python
            if obj.platform is not None:
                merged["platform"] = obj.platform
            return {key: normalize_payload(value) for key, value in merged.items()}
        payload: dict[str, Any] = {}
        for field in fields(obj):
            value = getattr(obj, field.name)
            if value is None:
                continue
            payload[field.name] = normalize_payload(value)
        return payload
    if isinstance(obj, Enum):
        return obj.value
    if isinstance(obj, dict):
        return {key: normalize_payload(value) for key, value in obj.items()}
    if isinstance(obj, list | tuple | set):
        return [normalize_payload(value) for value in obj]
    return obj


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
        log_policy=effective.log_policy,
    )
    return effective, output_format


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
        log_policy=resolved.log_policy,
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
        payload=normalize_payload(payload),
        fmt=output_format,
        pretty=effective_pretty,
        show_traceback=False,
    )
    raise ExitIntentError(intent)


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
    """Resolve error behavior and raise an exit intent."""
    behavior = resolve_exit_behavior(
        error_type or ErrorType.INTERNAL,
        quiet=quiet,
        fmt=fmt or OutputFormat.JSON,
        log_policy=log_policy,
    )

    error_payload: dict[str, Any] = {"error": message, "code": int(code)}
    if failure:
        error_payload["failure"] = failure
    if command:
        error_payload["command"] = command
    if fmt:
        error_payload["fmt"] = fmt
    if extra:
        error_payload.update(extra)
    if behavior.show_traceback:
        import traceback

        trace = traceback.format_exc()
        if "NoneType: None" not in trace:
            error_payload["traceback"] = trace
    if include_runtime:
        error_payload["python"] = ascii_safe(sys.version.split()[0], "python_version")
        error_payload["platform"] = ascii_safe(sys.platform, "platform")
        error_payload["timestamp"] = str(time.time())

    intent = ExitIntent(
        code=ExitCode(int(code)),
        stream=behavior.stream,
        payload=normalize_payload(error_payload),
        fmt=fmt or OutputFormat.JSON,
        pretty=False,
        show_traceback=behavior.show_traceback,
    )
    raise ExitIntentError(intent)


__all__ = [
    "current_execution_policy",
    "emit_error_with_policy",
    "new_run_command",
    "normalize_payload",
    "record_history",
    "resolve_command_config",
]
