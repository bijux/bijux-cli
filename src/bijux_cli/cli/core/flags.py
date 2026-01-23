# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Global CLI flag parsing helpers."""

from __future__ import annotations

from bijux_cli.cli.core.constants import (
    OPT_COLOR,
    OPT_FORMAT,
    OPT_HELP,
    OPT_LOG_LEVEL,
    OPT_QUIET,
)
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import FlagError, FlagLayer, GlobalCLIConfig


def parse_global_flags(argv: list[str]) -> GlobalCLIConfig:
    """Parse global CLI flags from argv into an immutable config."""
    help_flag = False
    suppress_errors = False
    quiet: bool | None = None
    log_level: LogLevel | None = None
    color: ColorMode | None = None
    fmt: OutputFormat | None = None
    errors: list[FlagError] = []
    retained: list[str] = list(argv)
    i = 0
    while i < len(argv):
        flag = argv[i]
        if flag in OPT_HELP:
            help_flag = True
            suppress_errors = True
            i += 1
            continue
        if flag in OPT_QUIET:
            quiet = True
            i += 1
            continue
        if flag in OPT_LOG_LEVEL:
            try:
                log_level = LogLevel(argv[i + 1])
                i += 2
            except IndexError:
                errors.append(
                    FlagError(
                        message="Missing value for --log-level.",
                        failure="missing_argument",
                        flag="--log-level",
                    )
                )
                i += 1
            except ValueError:
                errors.append(
                    FlagError(
                        message="Invalid log level.",
                        failure="invalid_log_level",
                        flag="--log-level",
                    )
                )
                i += 2
            continue
        if flag in OPT_COLOR:
            try:
                color = ColorMode(argv[i + 1])
                i += 2
            except IndexError:
                errors.append(
                    FlagError(
                        message="Missing value for --color.",
                        failure="missing_argument",
                        flag="--color",
                    )
                )
                i += 1
            except ValueError:
                errors.append(
                    FlagError(
                        message="Invalid color mode.",
                        failure="invalid_color",
                        flag="--color",
                    )
                )
                i += 2
            continue
        if flag in OPT_FORMAT:
            try:
                raw_value = argv[i + 1]
                fmt = OutputFormat(raw_value)
                i += 2
            except IndexError:
                errors.append(
                    FlagError(
                        message="Missing value for --format.",
                        failure="missing_argument",
                        flag="--format",
                    )
                )
                i += 1
            except ValueError:
                errors.append(
                    FlagError(
                        message=f"Unsupported format: {argv[i + 1]}",
                        failure="invalid_format",
                        flag="--format",
                    )
                )
                i += 2
            continue
        i += 1
    if suppress_errors:
        errors = []
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
