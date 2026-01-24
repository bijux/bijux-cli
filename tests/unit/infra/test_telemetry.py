# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra telemetry module."""

from __future__ import annotations

from typing import Any

from bijux_cli.core.enums import LogLevel
from bijux_cli.infra.telemetry import LoggingTelemetry, NoopTelemetry, TelemetryEvent


class DummyObs:
    """A mock observability service that records log calls."""

    def __init__(self) -> None:
        """Initialize the dummy observer."""
        self.calls: list[tuple[str, str, dict[str, Any] | None]] = []

    def log(self, level: str, msg: str, extra: dict[str, Any] | None = None) -> None:
        """Record a log call's parameters."""
        self.calls.append((level, msg, extra))


def test_nulltelemetry_methods_accept_all_args_and_do_nothing() -> None:
    """Test that NoopTelemetry methods are no-ops and do not raise errors."""
    nt = NoopTelemetry()
    nt.event("foo", {"bar": 1})
    nt.event(TelemetryEvent.CLI_STARTED, {"baz": 2})
    nt.flush()
    nt.enable()


def test_loggingtelemetry_event_string_and_enum() -> None:
    """Test that LoggingTelemetry correctly logs events with string and enum names."""
    from typing import cast

    from bijux_cli.services.contracts import ObservabilityProtocol

    obs = DummyObs()
    tel = LoggingTelemetry(cast(ObservabilityProtocol, obs))

    tel.event("custom_evt", {"x": 123})
    assert obs.calls[-1][0] == LogLevel.DEBUG
    assert "custom_evt" in obs.calls[-1][1]
    assert obs.calls[-1][2]
    assert obs.calls[-1][2]["x"] == 123
    assert tel._buffer[-1][0] == "custom_evt"

    tel.event(TelemetryEvent.PLUGIN_STARTED, {"y": 456})
    assert obs.calls[-1][1].endswith("plugin_started")
    assert tel._buffer[-1][0] == "plugin_started"
    assert len(tel._buffer) == 2


def test_loggingtelemetry_flush_clears_buffer() -> None:
    """Test that flushing LoggingTelemetry clears its internal event buffer."""
    from typing import cast

    from bijux_cli.services.contracts import ObservabilityProtocol

    obs = DummyObs()
    tel = LoggingTelemetry(cast(ObservabilityProtocol, obs))
    tel.event("foo", {"a": 1})
    tel.event("bar", {"b": 2})
    assert len(tel._buffer) == 2
    tel.flush()
    assert not tel._buffer


def test_loggingtelemetry_enable_is_noop() -> None:
    """Test that the enable method on LoggingTelemetry is a no-op."""
    from typing import cast

    from bijux_cli.services.contracts import ObservabilityProtocol

    obs = DummyObs()
    tel = LoggingTelemetry(cast(ObservabilityProtocol, obs))
    tel.enable()
    tel.event("baz", {"z": 3})
    assert tel._buffer


def test_loggingtelemetry_inject_constructor() -> None:
    """Test the constructor of LoggingTelemetry, simulating DI injection."""
    from typing import cast

    from bijux_cli.services.contracts import ObservabilityProtocol

    obs = DummyObs()
    tel = (
        LoggingTelemetry.__wrapped__(LoggingTelemetry, obs)
        if hasattr(LoggingTelemetry, "__wrapped__")
        else LoggingTelemetry(cast(ObservabilityProtocol, obs))
    )
    assert isinstance(tel, LoggingTelemetry)
    tel.event("evt", {})
