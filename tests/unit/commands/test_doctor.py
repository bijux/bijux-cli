# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the doctor command."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

from typing import Any
from unittest.mock import MagicMock, patch

import pytest
from typer import Context

from bijux_cli.commands.doctor import _build_payload, doctor


def test_build_payload_path_empty(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a missing PATH environment variable is detected as unhealthy."""
    monkeypatch.delenv("PATH", raising=False)
    monkeypatch.delenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", raising=False)

    payload: dict[str, Any] = _build_payload(include_runtime=False)  # type: ignore[assignment]
    assert payload["status"] == "unhealthy"
    assert "Environment PATH is empty" in payload["summary"]


def test_build_payload_force_unhealthy(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the unhealthy status can be forced via an environment variable."""
    monkeypatch.setenv("PATH", "/usr/bin")
    monkeypatch.setenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", "1")

    payload: dict[str, Any] = _build_payload(include_runtime=False)  # type: ignore[assignment]
    assert payload["status"] == "unhealthy"
    assert "Forced unhealthy by test environment" in payload["summary"]


def test_build_payload_combined_issues(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the payload correctly reports multiple combined issues."""
    monkeypatch.delenv("PATH", raising=False)
    monkeypatch.setenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", "1")

    payload: dict[str, Any] = _build_payload(include_runtime=True)  # type: ignore[assignment]
    assert payload["status"] == "unhealthy"
    assert "Environment PATH is empty" in payload["summary"]
    assert "Forced unhealthy by test environment" in payload["summary"]
    assert "python" in payload
    assert "platform" in payload


def test_build_payload_all_healthy(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the payload structure for a healthy system with and without runtime info."""
    monkeypatch.setenv("PATH", "/usr/bin")
    monkeypatch.delenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", raising=False)

    payload = _build_payload(include_runtime=False)
    assert payload["status"] == "healthy"
    assert payload["summary"] == ["All core checks passed"]
    assert "python" not in payload
    assert "platform" not in payload

    payload_rt: dict[str, Any] = _build_payload(include_runtime=True)  # type: ignore[assignment]
    assert payload_rt["status"] == "healthy"
    assert payload_rt["summary"] == ["All core checks passed"]
    assert "python" in payload_rt
    assert isinstance(payload_rt["python"], str)
    assert "platform" in payload_rt
    assert isinstance(payload_rt["platform"], str)


def test_build_payload_detects_empty_path_and_forced_unhealthy(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that both empty PATH and forced unhealthy states are correctly detected."""
    monkeypatch.delenv("PATH", raising=False)
    monkeypatch.delenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", raising=False)
    p1: dict[str, Any] = _build_payload(False)  # type: ignore[assignment]
    assert p1["status"] == "unhealthy"
    assert any("Environment PATH is empty" in msg for msg in p1["summary"])

    monkeypatch.setenv("PATH", "/usr/bin")
    monkeypatch.setenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", "1")
    p2: dict[str, Any] = _build_payload(False)  # type: ignore[assignment]
    assert p2["status"] == "unhealthy"
    assert any("Forced unhealthy by test environment" in msg for msg in p2["summary"])


def test_doctor_short_circuits_if_subcommand_set() -> None:
    """Test that the doctor command returns early if a subcommand is invoked."""
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = "anything"
    result = doctor(
        ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False
    )
    assert result is None


def test_doctor_di_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    """Handle DI resolution failure in `doctor`."""
    monkeypatch.setenv("PATH", "/usr/bin")

    monkeypatch.setattr(
        "bijux_cli.commands.doctor.validate_common_flags",
        lambda f, c, q: f,
        raising=False,
    )

    fake_di = MagicMock()
    fake_di.resolve.side_effect = Exception("boom")
    monkeypatch.setattr(
        "bijux_cli.commands.doctor.DIContainer.current",
        lambda: fake_di,
        raising=False,
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []

    with patch("bijux_cli.commands.doctor.emit_error_and_exit") as mock_emit:
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            doctor(
                ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False
            )

        mock_emit.assert_called_once_with(
            "boom",
            code=1,
            failure="internal",
            command="doctor",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_doctor_success_path(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the successful execution path of the doctor command."""
    monkeypatch.setenv("PATH", "/usr/bin")
    monkeypatch.delenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", raising=False)
    monkeypatch.setattr(
        "bijux_cli.commands.doctor.validate_common_flags",
        lambda fmt, c, q: fmt,
        raising=False,
    )

    fake_di = MagicMock()
    fake_di.resolve.return_value = None
    monkeypatch.setattr(
        "bijux_cli.commands.doctor.DIContainer.current",
        lambda: fake_di,
        raising=False,
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []

    with patch("bijux_cli.commands.doctor.new_run_command") as mock_new:
        doctor(ctx, quiet=True, verbose=True, fmt="yaml", pretty=False, debug=True)

    mock_new.assert_called_once()
    kw = mock_new.call_args.kwargs
    assert kw["command_name"] == "doctor"
    builder = kw["payload_builder"]
    p0 = builder(False)
    assert "status" in p0
    assert "summary" in p0
    p1 = builder(True)
    assert "python" in p1
    assert "platform" in p1


@patch("bijux_cli.commands.doctor.emit_error_and_exit")
@patch("bijux_cli.commands.doctor.validate_common_flags", lambda fmt, cmd, q: fmt)
def test_doctor_stray_option_calls_emit_and_exits(mock_emit: MagicMock) -> None:
    """Test that a stray unknown option results in a structured error."""
    mock_emit.side_effect = SystemExit()

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = ["-x"]

    with pytest.raises(SystemExit):
        doctor(ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False)

    mock_emit.assert_called_once_with(
        "No such option: -x",
        code=2,
        failure="args",
        command="doctor",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


@patch("bijux_cli.commands.doctor.emit_error_and_exit")
@patch("bijux_cli.commands.doctor.validate_common_flags", lambda fmt, cmd, q: fmt)
def test_doctor_stray_argument_calls_emit_and_exits(mock_emit: MagicMock) -> None:
    """Test that a stray argument results in a structured error."""
    mock_emit.side_effect = SystemExit()

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = ["foo"]

    with pytest.raises(SystemExit):
        doctor(ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False)

    mock_emit.assert_called_once_with(
        "Too many arguments: foo",
        code=2,
        failure="args",
        command="doctor",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )
