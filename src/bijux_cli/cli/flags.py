# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Global CLI flag parsing helpers."""

from __future__ import annotations

from typing import Any

from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import FlagLayer, GlobalCLIConfig


def parse_global_flags(argv: list[str]) -> GlobalCLIConfig:
    """Parse global CLI flags from argv into an immutable config."""
    help_present = any(flag in ("-h", "--help") for flag in argv)
    help_flag = False
    quiet: bool | None = None
    log_level: LogLevel | None = None
    color: ColorMode | None = None
    fmt: OutputFormat | None = None
    errors: list[dict[str, Any]] = []
    retained: list[str] = []
    it = iter(argv)
    for flag in it:
        if flag in ("-h", "--help"):
            help_flag = True
            retained.append(flag)
            continue
        if help_present:
            retained.append(flag.lstrip("-"))
            continue
        if flag in ("-q", "--quiet"):
            quiet = True
        elif flag == "--log-level":
            try:
                log_level = LogLevel(next(it))
            except StopIteration:
                errors.append(
                    {
                        "message": "Missing value for --log-level.",
                        "failure": "missing_argument",
                        "flag": "--log-level",
                    }
                )
            except ValueError:
                errors.append(
                    {
                        "message": "Invalid log level.",
                        "failure": "invalid_log_level",
                        "flag": "--log-level",
                    }
                )
        elif flag == "--color":
            try:
                color = ColorMode(next(it))
            except StopIteration:
                errors.append(
                    {
                        "message": "Missing value for --color.",
                        "failure": "missing_argument",
                        "flag": "--color",
                    }
                )
            except ValueError:
                errors.append(
                    {
                        "message": "Invalid color mode.",
                        "failure": "invalid_color",
                        "flag": "--color",
                    }
                )
        elif flag == "--json":
            fmt = OutputFormat.JSON
        else:
            retained.append(flag)
    return GlobalCLIConfig(
        help=help_flag,
        flags=FlagLayer(
            quiet=quiet,
            log_level=log_level,
            color=color,
            format=fmt,
        ),
        args=tuple(retained),
        errors=tuple(errors),
    )


__all__ = ["parse_global_flags"]
