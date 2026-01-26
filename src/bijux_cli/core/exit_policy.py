# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Formal error-to-exit behavior mapping."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

from bijux_cli.core.enums import ErrorType, ExitCode, OutputFormat

if TYPE_CHECKING:
    from bijux_cli.core.precedence import LogPolicy


@dataclass(frozen=True)
class ExitBehavior:
    """Defines how an error should exit and where it should emit."""

    code: ExitCode
    stream: str | None
    show_traceback: bool


@dataclass(frozen=True)
class ExitIntent:
    """Represents an execution intent for exiting with optional output."""

    code: ExitCode
    stream: str | None
    payload: Any | None
    fmt: OutputFormat
    pretty: bool
    show_traceback: bool


class ExitIntentError(Exception):
    """Raised to signal an exit intent without performing the exit."""

    def __init__(self, intent: ExitIntent) -> None:
        """Store the exit intent payload."""
        super().__init__("exit_intent")
        self.intent = intent


_BASE_BEHAVIOR: dict[ErrorType, ExitBehavior] = {
    ErrorType.USAGE: ExitBehavior(ExitCode.USAGE, "stdout", False),
    ErrorType.ASCII: ExitBehavior(ExitCode.ASCII, "stderr", False),
    ErrorType.USER_INPUT: ExitBehavior(ExitCode.USAGE, "stderr", False),
    ErrorType.PLUGIN: ExitBehavior(ExitCode.ERROR, "stderr", True),
    ErrorType.CONFIG: ExitBehavior(ExitCode.ERROR, "stderr", False),
    ErrorType.INTERNAL: ExitBehavior(ExitCode.ERROR, "stderr", True),
    ErrorType.ABORTED: ExitBehavior(ExitCode.ABORTED, "stderr", False),
}
_EXPECTED_ERROR_TYPES = set(ErrorType)
if set(_BASE_BEHAVIOR) != _EXPECTED_ERROR_TYPES:
    missing = _EXPECTED_ERROR_TYPES - set(_BASE_BEHAVIOR)
    extra = set(_BASE_BEHAVIOR) - _EXPECTED_ERROR_TYPES
    raise RuntimeError(
        f"Exit policy is incomplete. missing={sorted(missing)} extra={sorted(extra)}"
    )


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


def resolve_error_type(code: int, explicit: ErrorType | None = None) -> ErrorType:
    """Resolve an error type from an explicit override or exit code."""
    if explicit is not None:
        return explicit
    if code == ExitCode.USAGE:
        return ErrorType.USAGE
    if code == ExitCode.ASCII:
        return ErrorType.ASCII
    if code == ExitCode.ABORTED:
        return ErrorType.ABORTED
    return ErrorType.INTERNAL


def resolve_error_behavior(
    code: int,
    *,
    quiet: bool,
    fmt: OutputFormat,
    log_policy: LogPolicy,
    error_type: ErrorType | None = None,
) -> ExitBehavior:
    """Resolve exit behavior for an error code and context."""
    resolved_type = resolve_error_type(code, error_type)
    return resolve_exit_behavior(
        resolved_type, quiet=quiet, fmt=fmt, log_policy=log_policy
    )


__all__ = [
    "ExitBehavior",
    "ExitIntent",
    "ExitIntentError",
    "resolve_error_type",
    "resolve_error_behavior",
    "resolve_exit_behavior",
]
