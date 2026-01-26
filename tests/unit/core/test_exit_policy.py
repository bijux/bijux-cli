# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

from __future__ import annotations

import pytest

from bijux_cli.core.enums import ErrorType, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import (
    resolve_error_behavior,
    resolve_error_type,
    resolve_exit_behavior,
)
from bijux_cli.core.precedence import resolve_log_policy


@pytest.mark.parametrize(
    ("error_type", "expected_code", "expected_stream"),
    [
        (ErrorType.USAGE, ExitCode.USAGE, "stdout"),
        (ErrorType.ASCII, ExitCode.ASCII, "stderr"),
        (ErrorType.USER_INPUT, ExitCode.USAGE, "stderr"),
        (ErrorType.CONFIG, ExitCode.ERROR, "stderr"),
        (ErrorType.PLUGIN, ExitCode.ERROR, "stderr"),
        (ErrorType.INTERNAL, ExitCode.ERROR, "stderr"),
        (ErrorType.ABORTED, ExitCode.ABORTED, "stderr"),
    ],
)
def test_exit_behavior_matrix(
    error_type: ErrorType, expected_code: ExitCode, expected_stream: str
) -> None:
    behavior = resolve_exit_behavior(
        error_type,
        quiet=False,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
    )
    assert behavior.code is expected_code
    assert behavior.stream == expected_stream


@pytest.mark.parametrize("error_type", list(ErrorType))
def test_exit_behavior_quiet_suppresses_output(error_type: ErrorType) -> None:
    behavior = resolve_exit_behavior(
        error_type,
        quiet=True,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
    )
    assert behavior.stream is None


def test_unknown_exit_code_defaults_internal() -> None:
    assert resolve_error_type(999) is ErrorType.INTERNAL


def test_known_exit_codes_map_types() -> None:
    assert resolve_error_type(ExitCode.USAGE) is ErrorType.USAGE
    assert resolve_error_type(ExitCode.ASCII) is ErrorType.ASCII
    assert resolve_error_type(ExitCode.ABORTED) is ErrorType.ABORTED


def test_error_behavior_honors_quiet_and_json() -> None:
    behavior = resolve_error_behavior(
        1,
        quiet=True,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
    )
    assert behavior.code is ExitCode.ERROR
    assert behavior.stream is None


def test_error_behavior_respects_explicit_type() -> None:
    behavior = resolve_error_behavior(
        ExitCode.USAGE,
        quiet=False,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
        error_type=ErrorType.CONFIG,
    )
    assert behavior.code is ExitCode.ERROR


def test_traceback_visibility_depends_on_log_policy() -> None:
    behavior_info = resolve_exit_behavior(
        ErrorType.INTERNAL,
        quiet=False,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
    )
    behavior_debug = resolve_exit_behavior(
        ErrorType.INTERNAL,
        quiet=False,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.DEBUG),
    )
    assert behavior_info.show_traceback is False
    assert behavior_debug.show_traceback is True
