# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Shared validation helpers for CLI flags and environment."""

from __future__ import annotations

import os
from pathlib import Path
import re
import sys
import time
from typing import Any

from bijux_cli.cli.core.constants import ENV_CONFIG, ENV_PREFIX
from bijux_cli.core.enums import ErrorType, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import (
    ExitIntent,
    ExitIntentError,
    resolve_exit_behavior,
)
from bijux_cli.core.precedence import LogPolicy, resolve_log_policy

_ALLOWED_CTRL = {"\n", "\r", "\t"}
_ENV_LINE_RX = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=[A-Za-z0-9_./\\-]*$")


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
    log_policy: LogPolicy | None = None,
) -> OutputFormat:
    """Validate output format and ASCII environment."""

    def _raise_error_intent(
        message: str,
        *,
        code: ExitCode,
        failure: str,
        fmt_value: OutputFormat,
        behavior_stream: str | None,
        show_traceback: bool,
    ) -> None:
        error_payload = {"error": message, "code": int(code)}
        if failure:
            error_payload["failure"] = failure
        error_payload["command"] = command
        error_payload["fmt"] = fmt_value
        if show_traceback:
            import traceback

            trace = traceback.format_exc()
            if "NoneType: None" not in trace:
                error_payload["traceback"] = trace
        if include_runtime:
            error_payload["python"] = ascii_safe(
                sys.version.split()[0], "python_version"
            )
            error_payload["platform"] = ascii_safe(sys.platform, "platform")
            error_payload["timestamp"] = str(time.time())

        intent = ExitIntent(
            code=code,
            stream=behavior_stream,
            payload=error_payload,
            fmt=fmt_value,
            pretty=False,
            show_traceback=show_traceback,
        )
        raise ExitIntentError(intent)

    format_value = normalize_format(fmt)
    if format_value is None:
        format_value = OutputFormat.JSON
        behavior = resolve_exit_behavior(
            ErrorType.USAGE,
            quiet=quiet,
            fmt=OutputFormat.JSON,
            log_policy=log_policy or resolve_log_policy(LogLevel.INFO),
        )
        _raise_error_intent(
            f"Unsupported format: {fmt}",
            code=behavior.code,
            failure="format",
            fmt_value=OutputFormat.JSON,
            behavior_stream=behavior.stream,
            show_traceback=behavior.show_traceback,
        )
    if format_value not in (OutputFormat.JSON, OutputFormat.YAML):
        behavior = resolve_exit_behavior(
            ErrorType.USAGE,
            quiet=quiet,
            fmt=format_value,
            log_policy=log_policy or resolve_log_policy(LogLevel.INFO),
        )
        _raise_error_intent(
            f"Unsupported format: {fmt}",
            code=behavior.code,
            failure="format",
            fmt_value=format_value,
            behavior_stream=behavior.stream,
            show_traceback=behavior.show_traceback,
        )

    if contains_non_ascii_env():
        behavior = resolve_exit_behavior(
            ErrorType.ASCII,
            quiet=quiet,
            fmt=format_value,
            log_policy=log_policy or resolve_log_policy(LogLevel.INFO),
        )
        _raise_error_intent(
            "Non-ASCII in configuration or environment",
            code=behavior.code,
            failure="ascii",
            fmt_value=format_value,
            behavior_stream=behavior.stream,
            show_traceback=behavior.show_traceback,
        )

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
    "normalize_format",
    "contains_non_ascii_env",
    "validate_common_flags",
    "validate_env_file_if_present",
]
