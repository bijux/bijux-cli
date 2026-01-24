# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Global CLI flag parsing helpers."""

from __future__ import annotations

from dataclasses import dataclass

from bijux_cli.cli.core.constants import (
    OPT_COLOR,
    OPT_FORMAT,
    OPT_LOG_LEVEL,
    OPT_QUIET,
)
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import FlagError


@dataclass(frozen=True)
class Flags:
    """Parsed global CLI flags without precedence or policy decisions."""

    quiet: bool | None = None
    log_level: LogLevel | None = None
    color: ColorMode | None = None
    format: OutputFormat | None = None


def parse_global_flags(argv: list[str]) -> Flags:
    """Parse global CLI flags into a data-only Flags bundle."""
    quiet: bool | None = None
    log_level: LogLevel | None = None
    color: ColorMode | None = None
    fmt: OutputFormat | None = None
    i = 0
    while i < len(argv):
        flag = argv[i]
        if flag in OPT_QUIET:
            quiet = True
            i += 1
            continue
        if flag in OPT_LOG_LEVEL:
            try:
                log_level = LogLevel(argv[i + 1])
                i += 2
            except (IndexError, ValueError):
                i += 1
            continue
        if flag in OPT_COLOR:
            try:
                color = ColorMode(argv[i + 1])
                i += 2
            except (IndexError, ValueError):
                i += 1
            continue
        if flag in OPT_FORMAT:
            try:
                fmt = OutputFormat(argv[i + 1])
                i += 2
            except (IndexError, ValueError):
                i += 1
            continue
        i += 1
    return Flags(
        quiet=quiet,
        log_level=log_level,
        color=color,
        format=fmt,
    )


def collect_global_flag_errors(argv: list[str]) -> tuple[FlagError, ...]:
    """Collect parse-time flag errors without applying precedence or defaults."""
    errors: list[FlagError] = []
    i = 0
    while i < len(argv):
        flag = argv[i]
        if flag in OPT_LOG_LEVEL:
            try:
                LogLevel(argv[i + 1])
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
                ColorMode(argv[i + 1])
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
                OutputFormat(argv[i + 1])
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
    return tuple(errors)


__all__ = ["Flags", "collect_global_flag_errors", "parse_global_flags"]
