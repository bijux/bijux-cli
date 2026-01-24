# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

from __future__ import annotations

import pytest

from bijux_cli.core.enums import ErrorType, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import resolve_exit_behavior
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
