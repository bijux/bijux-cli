# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra observability module."""

from __future__ import annotations

from typing import Any, cast
from unittest.mock import MagicMock

import pytest
from structlog.typing import FilteringBoundLogger

from bijux_cli.contracts import TelemetryProtocol
from bijux_cli.core.exceptions import ServiceError
from bijux_cli.infra.observability import Observability
from bijux_cli.infra.telemetry import NullTelemetry

# pyright: reportPrivateUsage=false


def test_setup_and_defaults() -> None:
    """Test the setup and default state of the Observability service."""
    obs = Observability.setup()
    assert isinstance(obs, Observability)
    assert isinstance(obs._telemetry, NullTelemetry)
    assert obs.get_logger() is obs._logger


def test_set_telemetry_is_chainable_and_sets_backend() -> None:
    """Test that setting a telemetry backend is chainable and correctly assigned."""
    obs = Observability.setup()
    telemetry = MagicMock(spec=TelemetryProtocol)

    returned = obs.set_telemetry(telemetry)

    assert returned is obs
    assert obs._telemetry is telemetry


def test_bind_replaces_logger_and_is_chainable() -> None:
    """Test that binding context to the logger returns a new logger and is chainable."""
    obs = Observability.setup()

    original_logger = MagicMock()
    bound_logger = MagicMock()
    original_logger.bind.return_value = bound_logger
    obs._logger = original_logger

    returned = obs.bind(user="alice", request_id="123")

    original_logger.bind.assert_called_once_with(user="alice", request_id="123")
    assert obs._logger is bound_logger
    assert returned is obs


def test_log_with_extra_calls_correct_level_and_emits_telemetry() -> None:
    """Test that logging with extra data calls the correct logger method and emits telemetry."""
    obs = Observability.setup()

    logger = MagicMock()
    obs._logger = logger
    telemetry = MagicMock(spec=TelemetryProtocol)
    obs.set_telemetry(telemetry)

    returned = obs.log("INFO", "hello", extra={"foo": "bar"})

    logger.info.assert_called_once_with("hello", foo="bar")
    telemetry.event.assert_called_once_with(
        "LOG_EMITTED", {"level": "INFO", "message": "hello", "foo": "bar"}
    )
    assert returned is obs


def test_log_without_extra_calls_level_and_emits_minimal_telemetry() -> None:
    """Test that logging without extra data emits a minimal telemetry event."""
    obs = Observability.setup()

    logger = MagicMock()
    obs._logger = logger
    telemetry = MagicMock(spec=TelemetryProtocol)
    obs.set_telemetry(telemetry)

    obs.log("debug", "dbg")

    logger.debug.assert_called_once_with("dbg")
    telemetry.event.assert_called_once_with(
        "LOG_EMITTED", {"level": "debug", "message": "dbg"}
    )


def test_log_with_nulltelemetry_skips_telemetry_event() -> None:
    """Test that no telemetry event is emitted when using NullTelemetry."""
    obs = Observability.setup()
    logger = MagicMock()
    obs._logger = logger

    obs.log("info", "just-logs")

    logger.info.assert_called_once_with("just-logs")


def test_log_invalid_level_raises_service_error() -> None:
    """Tests that logging with an invalid level raises a ServiceError."""
    obs = Observability.setup()

    class DummyLogger:
        def bind(self, **kwargs: Any) -> DummyLogger:  # pytype: disable=name-error
            """A dummy bind method that returns itself."""
            return self

    obs._logger = cast(FilteringBoundLogger, DummyLogger())
    with pytest.raises(ServiceError, match="Invalid log level: BOGUS"):
        obs.log("BOGUS", "nope")


def test_close_calls_debug_shutdown() -> None:
    """Test that the close method logs a shutdown message."""
    obs = Observability.setup()
    logger = MagicMock()
    obs._logger = logger

    obs.close()

    logger.debug.assert_called_once_with("Observability shutdown")
