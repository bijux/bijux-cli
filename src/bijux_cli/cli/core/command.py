# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Command helpers for policy-aware execution and exit intents."""

from __future__ import annotations

import os
from pathlib import Path
import re
import sys
from typing import Any, NoReturn

from bijux_cli.cli.core.constants import ENV_CONFIG, ENV_PREFIX
from bijux_cli.core.enums import ErrorType, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError
from bijux_cli.core.precedence import current_execution_policy, resolve_exit_intent
from bijux_cli.infra.contracts import Emitter, Serializer

_ALLOWED_CTRL = {"\n", "\r", "\t"}
_ENV_LINE_RX = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=[A-Za-z0-9_./\\-]*$")


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


def resolve_serializer() -> Serializer:
    """Resolve the serializer adapter."""
    from bijux_cli.core.di import DIContainer

    serializer = DIContainer.current().resolve(Serializer)
    if not hasattr(serializer, "dumps"):
        raise RuntimeError("Serializer does not implement dumps()")
    return serializer


def resolve_emitter() -> Emitter | None:
    """Resolve the emitter adapter or return None."""
    from bijux_cli.core.di import DIContainer

    try:
        return DIContainer.current().resolve(Emitter)
    except Exception:
        return None


def emit_payload(
    payload: object,
    *,
    serializer: Serializer,
    emitter: Emitter | None,
    fmt: OutputFormat,
    pretty: bool,
    stream: str,
) -> None:
    """Emit a payload to the requested stream."""
    out = sys.stdout if stream == "stdout" else sys.stderr
    _ = emitter
    output = serializer.dumps(payload, fmt=fmt, pretty=pretty).rstrip("\n")
    print(output, file=out, flush=True)


def ascii_safe(text: Any, _field: str = "") -> str:
    """Return a printable ASCII-only string."""
    text_str = text if isinstance(text, str) else str(text)
    return "".join(
        ch if (32 <= ord(ch) <= 126) or ch in _ALLOWED_CTRL else "?" for ch in text_str
    )


def normalize_format(fmt: str | OutputFormat | None) -> OutputFormat | None:
    """Normalize a format value into OutputFormat."""
    if isinstance(fmt, OutputFormat):
        return fmt
    if isinstance(fmt, str):
        value = fmt.strip().lower()
        if value in ("json", "yaml"):
            return OutputFormat(value)
    return None


def contains_non_ascii_env() -> bool:
    """Return True when config env or file contents are non-ASCII."""
    config_path_str = os.environ.get(ENV_CONFIG)
    if config_path_str:
        if not config_path_str.isascii():
            return True
        try:
            config_path = Path(config_path_str)
        except NotImplementedError:
            return False
        if config_path.exists():
            try:
                config_path.read_text(encoding="ascii")
            except UnicodeDecodeError:
                return True
            except (IsADirectoryError, PermissionError, FileNotFoundError, OSError):
                pass

    for k, v in os.environ.items():
        if k.startswith(ENV_PREFIX) and not v.isascii():
            return True
    return False


def validate_common_flags(
    fmt: str | OutputFormat,
    command: str,
    quiet: bool,
    include_runtime: bool = False,
    log_level: LogLevel = LogLevel.INFO,
) -> OutputFormat:
    """Validate output format and ASCII environment."""
    format_value = normalize_format(fmt)
    if format_value is None:
        intent = resolve_exit_intent(
            message=f"Unsupported format: {fmt}",
            code=ExitCode.USAGE,
            failure="format",
            command=command,
            fmt=OutputFormat.JSON,
            quiet=quiet,
            include_runtime=include_runtime,
            error_type=ErrorType.USAGE,
            log_level=log_level,
        )
        raise ExitIntentError(intent)
    if format_value not in (OutputFormat.JSON, OutputFormat.YAML):
        intent = resolve_exit_intent(
            message="Invalid output format.",
            code=ExitCode.USAGE,
            failure="format",
            command=command,
            fmt=format_value,
            quiet=quiet,
            include_runtime=include_runtime,
            error_type=ErrorType.USAGE,
            log_level=log_level,
        )
        raise ExitIntentError(intent)

    if contains_non_ascii_env():
        intent = resolve_exit_intent(
            message="Non-ASCII in configuration or environment",
            code=ExitCode.ASCII,
            failure="ascii",
            command=command,
            fmt=format_value,
            quiet=quiet,
            include_runtime=include_runtime,
            error_type=ErrorType.ASCII,
            log_level=log_level,
        )
        raise ExitIntentError(intent)

    return format_value


def validate_env_file_if_present(path_str: str) -> None:
    """Validate env file format if present."""
    if not path_str or not Path(path_str).exists():
        return
    try:
        text = Path(path_str).read_text(encoding="utf-8", errors="strict")
    except Exception as exc:
        raise ValueError(f"Cannot read config file: {exc}") from exc

    for i, line in enumerate(text.splitlines(), start=1):
        s = line.strip()
        if s and not s.startswith("#") and not _ENV_LINE_RX.match(s):
            raise ValueError(f"Malformed line {i} in config: {line!r}")


__all__ = [
    "ascii_safe",
    "contains_non_ascii_env",
    "emit_payload",
    "new_run_command",
    "normalize_format",
    "record_history",
    "resolve_emitter",
    "resolve_serializer",
    "raise_exit_intent",
    "validate_common_flags",
    "validate_env_file_if_present",
]
