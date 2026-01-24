# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Emit structured payloads and exit."""

from __future__ import annotations

from dataclasses import fields, is_dataclass
from enum import Enum
import sys
import time
from typing import Any, NoReturn

import typer

from bijux_cli.cli.core.validation import ascii_safe
from bijux_cli.core.enums import ExitCode, OutputFormat
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError
from bijux_cli.infra.contracts import Serializer


def resolve_serializer() -> Serializer:
    """Resolve the serializer adapter or fallback."""
    import json

    from bijux_cli.core.di import DIContainer

    class _FallbackSerializer:
        """Minimal JSON serializer when DI is unavailable."""

        def dumps(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> str:
            _ = fmt
            return json.dumps(obj, indent=2 if pretty else None, default=str)

        def dumps_bytes(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> bytes:
            return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

        def loads(self, data: str | bytes, *, fmt: OutputFormat, pretty: bool) -> Any:
            _ = (fmt, pretty)
            if isinstance(data, bytes):
                data = data.decode("utf-8")
            return json.loads(data)

        def emit(self, payload: Any, *, fmt: OutputFormat, pretty: bool) -> None:
            print(self.dumps(payload, fmt=fmt, pretty=pretty).rstrip("\n"))

    try:
        serializer = DIContainer.current().resolve(Serializer)
        if not hasattr(serializer, "dumps"):
            raise RuntimeError("Serializer does not implement dumps()")
        return serializer
    except Exception:
        return _FallbackSerializer()


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


def emit_payload(
    payload: object, *, fmt: OutputFormat, pretty: bool, stream: str
) -> None:
    """Emit a payload to the requested stream."""
    out = sys.stdout if stream == "stdout" else sys.stderr
    try:
        output = (
            resolve_serializer().dumps(payload, fmt=fmt, pretty=pretty).rstrip("\n")
        )
        print(output, file=out, flush=True)
    except Exception:
        print('{"error": "Unserializable error"}', file=sys.stderr, flush=True)


def emit_and_exit(
    payload: object,
    fmt: OutputFormat,
    effective_pretty: bool,
    verbose: bool,
    command: str,
    *,
    exit_code: int = 0,
) -> NoReturn:
    """Build an exit intent for a successful command."""
    _ = verbose
    if command != "history":
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

    intent = ExitIntent(
        code=ExitCode(exit_code),
        stream="stdout",
        payload=_normalize_payload(payload),
        fmt=fmt,
        pretty=effective_pretty,
        show_traceback=False,
    )
    raise ExitIntentError(intent)


def emit_error_and_exit(
    message: str,
    code: int,
    failure: str,
    command: str | None = None,
    fmt: OutputFormat | None = None,
    include_runtime: bool = False,
    extra: dict[str, Any] | None = None,
    *,
    stream: str | None,
    show_traceback: bool,
) -> NoReturn:
    """Build an exit intent for an error payload."""
    error_payload = {"error": message, "code": int(code)}
    if failure:
        error_payload["failure"] = failure
    if command:
        error_payload["command"] = command
    if fmt:
        error_payload["fmt"] = fmt
    if extra:
        error_payload.update(extra)
    if show_traceback:
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
        stream=stream,
        payload=_normalize_payload(error_payload),
        fmt=fmt or OutputFormat.JSON,
        pretty=False,
        show_traceback=show_traceback,
    )
    raise ExitIntentError(intent)


def emit_text_and_exit(
    text: str,
    *,
    color: bool | None,
    stream: str = "stdout",
    exit_code: int = 0,
) -> NoReturn:
    """Emit plain text output and raise an exit intent."""
    typer.echo(text, color=color, err=(stream == "stderr"))
    intent = ExitIntent(
        code=ExitCode(exit_code),
        stream=None,
        payload=None,
        fmt=OutputFormat.JSON,
        pretty=False,
        show_traceback=False,
    )
    raise ExitIntentError(intent)


def exit_if_quiet(quiet: bool, code: int = 0) -> None:
    """Exit immediately when quiet mode suppresses output."""
    if quiet:
        import typer

        raise typer.Exit(code)


__all__ = [
    "emit_and_exit",
    "emit_error_and_exit",
    "emit_text_and_exit",
    "exit_if_quiet",
    "resolve_serializer",
]
