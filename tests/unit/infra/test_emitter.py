# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra emitter module."""

from __future__ import annotations

import sys
from typing import cast
from unittest.mock import MagicMock, Mock, mock_open, patch

import pytest

from bijux_cli.core.enums import LogLevel, OutputFormat
from bijux_cli.infra.emitter import ConsoleEmitter
from bijux_cli.services.contracts import TelemetryProtocol


@pytest.fixture
def mock_telemetry() -> MagicMock:
    """Provide a mock of the TelemetryProtocol."""
    return MagicMock(spec=TelemetryProtocol)


@pytest.fixture
def emitter(mock_telemetry: MagicMock) -> ConsoleEmitter:
    """Provide an ConsoleEmitter instance initialized with a mock telemetry service."""
    return ConsoleEmitter(mock_telemetry)


def test_init(mock_telemetry: MagicMock) -> None:
    """Test the ConsoleEmitter's constructor with all parameters."""
    em = ConsoleEmitter(mock_telemetry, output_format=OutputFormat.YAML)
    assert em._telemetry is mock_telemetry
    assert em._default_format == OutputFormat.YAML


@patch("bijux_cli.infra.emitter.serializer_for")
@patch("builtins.print")
def test_emit_stdout_success(
    mock_print: MagicMock, mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test a successful emission to stdout."""
    mock_serializer = MagicMock()
    mock_serializer.dumps.return_value = '{"key": "value"}'
    mock_serializer_for.return_value = mock_serializer

    emitter.emit(
        {"key": "value"},
        fmt=OutputFormat.JSON,
        pretty=False,
        level=LogLevel.INFO,
        message="msg",
        output=None,
        test="context",
    )

    mock_serializer_for.assert_called_with(OutputFormat.JSON, emitter._telemetry)
    mock_serializer.dumps.assert_called_with(
        {"key": "value"}, fmt=OutputFormat.JSON, pretty=False
    )
    mock_print.assert_called_with('{"key": "value"}', file=sys.stdout, flush=True)
    cast(Mock, emitter._telemetry.event).assert_called_with(
        "output_emitted", {"format": "json", "size_chars": len('{"key": "value"}')}
    )


@patch("bijux_cli.infra.emitter.serializer_for")
@patch("builtins.print")
def test_emit_default_format(
    mock_print: MagicMock, mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test that the default format is used when no format is specified."""
    emitter._default_format = OutputFormat.YAML
    mock_serializer = MagicMock()
    mock_serializer.dumps.return_value = "key: value"
    mock_serializer_for.return_value = mock_serializer

    emitter.emit({"key": "value"})

    mock_serializer_for.assert_called_with(OutputFormat.YAML, emitter._telemetry)


@patch("bijux_cli.infra.emitter.serializer_for")
def test_emit_serialization_fail(
    mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test that a serialization failure is handled and raises a RuntimeError."""
    mock_serializer = MagicMock()
    mock_serializer.dumps.side_effect = Exception("fail")
    mock_serializer_for.return_value = mock_serializer

    with patch.object(emitter._logger, "error") as mock_log:  # noqa: SIM117
        with pytest.raises(RuntimeError, match="Serialization failed: fail"):
            emitter.emit({})
    mock_log.assert_called_with("Serialization failed", error="fail")


@patch("bijux_cli.infra.emitter.serializer_for")
@patch("builtins.print")
def test_emit_debug_print(
    mock_print: MagicMock, mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test that diagnostic information is printed when debug mode is enabled."""
    mock_serializer = MagicMock()
    mock_serializer.dumps.return_value = "output"
    mock_serializer_for.return_value = mock_serializer

    with patch.object(emitter._logger, "error") as mock_log:
        emitter.emit(
            {}, level=LogLevel.ERROR, message="test_msg", emit_diagnostics=True
        )
    mock_print.assert_any_call("output", file=sys.stdout, flush=True)
    mock_log.assert_not_called()


@patch("bijux_cli.infra.emitter.serializer_for")
def test_emit_quiet_skip(
    mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test that emission is skipped for info level when quiet mode is enabled."""
    emitter.emit({}, level=LogLevel.INFO, emit_output=False)
    mock_serializer_for.assert_not_called()


@patch("bijux_cli.infra.emitter.serializer_for")
@patch("builtins.print")
def test_emit_quiet_error_proceed(
    mock_print: MagicMock, mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test that emission proceeds for error level even in quiet mode."""
    mock_serializer = MagicMock()
    mock_serializer.dumps.return_value = "error"
    mock_serializer_for.return_value = mock_serializer

    emitter.emit({}, level=LogLevel.ERROR, emit_output=True)

    mock_print.assert_called_with("error", file=sys.stdout, flush=True)


@patch("bijux_cli.infra.emitter.serializer_for")
@patch("builtins.print")
def test_emit_quiet_critical_proceed(
    mock_print: MagicMock, mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test that emission proceeds for critical level even in quiet mode."""
    mock_serializer = MagicMock()
    mock_serializer.dumps.return_value = "critical"
    mock_serializer_for.return_value = mock_serializer

    emitter.emit({}, level=LogLevel.CRITICAL, emit_output=True)

    mock_print.assert_called_with("critical", file=sys.stdout, flush=True)


@patch("bijux_cli.infra.emitter.serializer_for")
@patch("builtins.print")
def test_emit_telemetry_fail_debug(
    mock_print: MagicMock, mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test that a telemetry failure is logged when debug mode is enabled."""
    mock_serializer = MagicMock()
    mock_serializer.dumps.return_value = "output"
    cast(Mock, mock_serializer_for).return_value = mock_serializer
    cast(Mock, emitter._telemetry.event).side_effect = Exception("tel fail")

    with patch.object(emitter._logger, "debug") as mock_log:
        emitter.emit({}, level=LogLevel.ERROR, emit_diagnostics=True)
    mock_log.assert_called_with("Telemetry failed", error="tel fail")


@patch("bijux_cli.infra.emitter.serializer_for")
@patch("builtins.print")
def test_emit_telemetry_fail_no_debug(
    mock_print: MagicMock, mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test that a telemetry failure is silently ignored when debug is disabled."""
    mock_serializer = MagicMock()
    mock_serializer.dumps.return_value = "output"
    mock_serializer_for.return_value = mock_serializer
    cast(Mock, emitter._telemetry.event).side_effect = Exception("tel fail")

    with patch.object(emitter._logger, "error") as mock_log:
        emitter.emit({}, level=LogLevel.ERROR, emit_diagnostics=False)
    mock_log.assert_not_called()


@patch("bijux_cli.infra.emitter.serializer_for")
@patch("builtins.open", new_callable=mock_open)
def test_emit_file_success(
    mock_file_open: MagicMock, mock_serializer_for: MagicMock, emitter: ConsoleEmitter
) -> None:
    """Test successful emission to a file."""
    mock_serializer = MagicMock()
    mock_serializer.dumps.return_value = "output"
    mock_serializer_for.return_value = mock_serializer

    emitter.emit({}, output="file.txt")

    mock_file_open.assert_called_with("file.txt", "w", encoding="utf-8")
    mock_file_open().write.assert_called_with("output")


@patch("sys.stdout.flush")
def test_flush(mock_flush: MagicMock, emitter: ConsoleEmitter) -> None:
    """Test that the flush method calls sys.stdout.flush."""
    emitter.flush()
    mock_flush.assert_called_once()
