# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Formal error-to-exit behavior mapping."""

from __future__ import annotations

from dataclasses import dataclass

from bijux_cli.core.enums import ErrorType, ExitCode, OutputFormat


@dataclass(frozen=True)
class ExitBehavior:
    """Defines how an error should exit and where it should emit."""

    code: ExitCode
    stream: str | None


_BASE_BEHAVIOR: dict[ErrorType, ExitBehavior] = {
    ErrorType.USAGE: ExitBehavior(ExitCode.USAGE, "stdout"),
    ErrorType.ASCII: ExitBehavior(ExitCode.ASCII, "stderr"),
    ErrorType.USER_INPUT: ExitBehavior(ExitCode.ERROR, "stderr"),
    ErrorType.PLUGIN: ExitBehavior(ExitCode.ERROR, "stderr"),
    ErrorType.CONFIG: ExitBehavior(ExitCode.ERROR, "stderr"),
    ErrorType.INTERNAL: ExitBehavior(ExitCode.ERROR, "stderr"),
    ErrorType.ABORTED: ExitBehavior(ExitCode.ABORTED, "stderr"),
}


def resolve_exit_behavior(
    error_type: ErrorType, *, quiet: bool, fmt: OutputFormat
) -> ExitBehavior:
    """Return the exit behavior for a given error type and output context."""
    _ = fmt
    base = _BASE_BEHAVIOR[error_type]
    if quiet:
        return ExitBehavior(base.code, None)
    return base


__all__ = ["ExitBehavior", "resolve_exit_behavior"]
