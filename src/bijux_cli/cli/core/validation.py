# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Shared validation helpers for CLI flags and environment."""

from __future__ import annotations

import os
from pathlib import Path
import re
from typing import Any

from bijux_cli.cli.core.constants import ENV_CONFIG, ENV_PREFIX
from bijux_cli.core.enums import OutputFormat

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
) -> OutputFormat:
    """Validate output format and ASCII environment."""
    format_value = normalize_format(fmt)
    if format_value is None:
        format_value = OutputFormat.JSON
        from bijux_cli.cli.core.emit import emit_error_and_exit

        emit_error_and_exit(
            f"Unsupported format: {fmt}",
            code=2,
            failure="format",
            command=command,
            fmt=OutputFormat.JSON,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=False,
        )
    if format_value not in (OutputFormat.JSON, OutputFormat.YAML):
        from bijux_cli.cli.core.emit import emit_error_and_exit

        emit_error_and_exit(
            f"Unsupported format: {fmt}",
            code=2,
            failure="format",
            command=command,
            fmt=format_value,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=False,
        )

    if contains_non_ascii_env():
        from bijux_cli.cli.core.emit import emit_error_and_exit

        emit_error_and_exit(
            "Non-ASCII in configuration or environment",
            code=3,
            failure="ascii",
            command=command,
            fmt=format_value,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=False,
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
