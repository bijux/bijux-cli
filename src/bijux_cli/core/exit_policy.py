# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Formal error-to-exit behavior mapping."""

from __future__ import annotations

from dataclasses import dataclass

from bijux_cli.core.enums import ErrorType, ExitCode, OutputFormat
from bijux_cli.core.precedence import LogPolicy


@dataclass(frozen=True)
class ExitBehavior:
    """Defines how an error should exit and where it should emit."""

    code: ExitCode
    stream: str | None
    show_traceback: bool


_BASE_BEHAVIOR: dict[ErrorType, ExitBehavior] = {
    ErrorType.USAGE: ExitBehavior(ExitCode.USAGE, "stdout", False),
    ErrorType.ASCII: ExitBehavior(ExitCode.ASCII, "stderr", False),
    ErrorType.USER_INPUT: ExitBehavior(ExitCode.USAGE, "stderr", False),
    ErrorType.PLUGIN: ExitBehavior(ExitCode.ERROR, "stderr", True),
    ErrorType.CONFIG: ExitBehavior(ExitCode.ERROR, "stderr", False),
    ErrorType.INTERNAL: ExitBehavior(ExitCode.ERROR, "stderr", True),
    ErrorType.ABORTED: ExitBehavior(ExitCode.ABORTED, "stderr", False),
}


def resolve_exit_behavior(
    error_type: ErrorType,
    *,
    quiet: bool,
    fmt: OutputFormat,
    log_policy: LogPolicy,
) -> ExitBehavior:
    """Return the exit behavior for a given error type and output context."""
    _ = fmt
    base = _BASE_BEHAVIOR[error_type]
    show_traceback = base.show_traceback and log_policy.show_traceback
    if quiet:
        return ExitBehavior(base.code, None, show_traceback)
    return ExitBehavior(base.code, base.stream, show_traceback)


__all__ = ["ExitBehavior", "resolve_exit_behavior"]
