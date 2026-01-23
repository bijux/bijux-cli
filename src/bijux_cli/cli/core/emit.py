# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Emit structured payloads and exit."""

from __future__ import annotations

from dataclasses import fields, is_dataclass
from enum import Enum
import sys
import time
from typing import Any, NoReturn

from bijux_cli.cli.core.validation import ascii_safe
from bijux_cli.core.enums import ErrorType, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import resolve_exit_behavior
from bijux_cli.core.precedence import LogPolicy, resolve_log_policy
from bijux_cli.infra.contracts import Serializer


def resolve_serializer() -> Serializer:
    """Resolve the serializer adapter or fallback."""
    from bijux_cli.core.di import DIContainer

    serializer = DIContainer.current().resolve(Serializer)
    if not hasattr(serializer, "dumps"):
        raise RuntimeError("Serializer does not implement dumps()")
    return serializer


def _normalize_payload(obj: Any) -> Any:
    """Normalize dataclasses and enums into plain serializable structures."""
    if is_dataclass(obj):
        from bijux_cli.cli.commands.payloads import ConfigDumpPayload

        if isinstance(obj, ConfigDumpPayload):
            merged: dict[str, Any] = dict(obj.entries)
            if obj.python is not None:
                merged["python"] = obj.python
            if obj.platform is not None:
                merged["platform"] = obj.platform
            return {key: _normalize_payload(value) for key, value in merged.items()}
        payload: dict[str, Any] = {}
        for field in fields(obj):
            value = getattr(obj, field.name)
            if value is None:
                continue
            payload[field.name] = _normalize_payload(value)
        return payload
    if isinstance(obj, Enum):
        return obj.value
    if isinstance(obj, dict):
        return {key: _normalize_payload(value) for key, value in obj.items()}
    if isinstance(obj, list | tuple | set):
        return [_normalize_payload(value) for value in obj]
    return obj


def emit_and_exit(
    payload: object,
    fmt: OutputFormat,
    effective_pretty: bool,
    verbose: bool,
    debug: bool,
    quiet: bool,
    command: str,
    *,
    exit_code: int = 0,
) -> NoReturn:
    """Serialize payload, record history, and exit."""
    if (not quiet) and (not command.startswith("history")):
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

    if quiet:
        sys.exit(exit_code)

    output = resolve_serializer().dumps(payload, fmt=fmt, pretty=effective_pretty)
    print(output.rstrip("\n"))
    sys.exit(exit_code)


def emit_error_and_exit(
    message: str,
    code: int,
    failure: str,
    command: str | None = None,
    fmt: OutputFormat | None = None,
    quiet: bool = False,
    include_runtime: bool = False,
    debug: bool = False,
    extra: dict[str, Any] | None = None,
    error_type: ErrorType | None = None,
    log_policy: LogPolicy | None = None,
) -> NoReturn:
    """Emit a structured error payload to stderr and exit."""
    inferred_type = error_type
    if inferred_type is None:
        if code == 2:
            inferred_type = ErrorType.USAGE
        elif code == 3:
            inferred_type = ErrorType.ASCII
        elif code == 130:
            inferred_type = ErrorType.ABORTED
        else:
            inferred_type = ErrorType.INTERNAL

    policy = log_policy or resolve_log_policy(
        LogLevel.DEBUG if debug else LogLevel.INFO
    )
    behavior = resolve_exit_behavior(
        inferred_type, quiet=quiet, fmt=fmt or OutputFormat.JSON, log_policy=policy
    )
    code = int(behavior.code)
    if behavior.stream is None:
        sys.exit(code)

    error_payload = {"error": message, "code": code}
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

    serializer = resolve_serializer()
    try:
        out_format = fmt or OutputFormat.JSON
        output = serializer.dumps(
            error_payload,
            fmt=out_format,
            pretty=False,
        ).rstrip("\n")
        stream = sys.stderr
        if behavior.stream == "stdout":
            stream = sys.stdout
        print(output, file=stream, flush=True)
    except Exception:
        print('{"error": "Unserializable error"}', file=sys.stderr, flush=True)
    sys.exit(code)


__all__ = ["emit_and_exit", "emit_error_and_exit", "resolve_serializer"]
