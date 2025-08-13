# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the core exceptions module."""

from __future__ import annotations

import pytest

from bijux_cli.core.exceptions import (
    BijuxError,
    CliTimeoutError,
    CommandError,
    ConfigError,
    ServiceError,
    ValidationError,
)


def test_bijux_error_initialization_with_message_only() -> None:
    """Verify BijuxError's default attributes when only a message is provided."""
    with pytest.raises(BijuxError) as excinfo:
        raise BijuxError("A base error occurred")

    err = excinfo.value
    assert str(err) == "A base error occurred"
    assert err.command is None
    assert err.http_status == 500


def test_bijux_error_initialization_with_all_attributes() -> None:
    """Verify all BijuxError attributes are set correctly when provided."""
    with pytest.raises(BijuxError) as excinfo:
        raise BijuxError("A specific error", command="test-cmd", http_status=418)

    err = excinfo.value
    assert str(err) == "A specific error"
    assert err.command == "test-cmd"
    assert err.http_status == 418


@pytest.mark.parametrize(
    ("error_class", "default_status"),
    [
        (ServiceError, 500),
        (CommandError, 400),
        (ConfigError, 400),
        (ValidationError, 400),
        (CliTimeoutError, 504),
    ],
)
def test_derived_exception_initialization(
    error_class: type[BijuxError], default_status: int
) -> None:
    """Verify initialization for all derived exception classes in various scenarios."""
    try:
        raise error_class("test")
    except BijuxError:
        pass
    except Exception:
        pytest.fail(f"{error_class.__name__} did not inherit from BijuxError")

    with pytest.raises(error_class) as excinfo_msg_only:
        raise error_class("A derived error occurred")
    err_msg_only = excinfo_msg_only.value
    assert str(err_msg_only) == "A derived error occurred"
    assert err_msg_only.command is None
    assert err_msg_only.http_status == default_status

    with pytest.raises(error_class) as excinfo_with_cmd:
        raise error_class("Error in command", command="run-task")
    err_with_cmd = excinfo_with_cmd.value
    assert err_with_cmd.command == "run-task"
    assert err_with_cmd.http_status == default_status

    with pytest.raises(error_class) as excinfo_with_status:
        raise error_class("Custom status error", http_status=429)
    err_with_status = excinfo_with_status.value
    assert err_with_status.command is None
    assert err_with_status.http_status == 429

    with pytest.raises(error_class) as excinfo_full:
        raise error_class("A fully specified error", command="do-it", http_status=451)
    err_full = excinfo_full.value
    assert str(err_full) == "A fully specified error"
    assert err_full.command == "do-it"
    assert err_full.http_status == 451


def test_all_exports_are_correct() -> None:
    """Verify that all custom exception classes are included in the module's __all__."""
    from bijux_cli.core import exceptions

    expected_exports = [
        "BijuxError",
        "ServiceError",
        "CommandError",
        "ConfigError",
        "ValidationError",
        "CliTimeoutError",
    ]
    assert sorted(exceptions.__all__) == sorted(expected_exports)
