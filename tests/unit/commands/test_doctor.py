# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the doctor command."""

from __future__ import annotations

from unittest.mock import MagicMock, patch

import pytest
from typer import Context

from bijux_cli.cli.commands.diagnostics.doctor import _build_payload, doctor
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import ExecutionPolicy


def _fake_resolve_command_config(
    **kwargs: object,
) -> tuple[ExecutionPolicy, OutputFormat, OutputFormat]:
    fmt = str(kwargs.get("fmt") or "json").lower()
    output_format = OutputFormat.YAML if fmt == "yaml" else OutputFormat.JSON
    verbose = bool(kwargs.get("verbose", False))
    log_level_raw = kwargs.get("log_level", LogLevel.INFO)
    log_level = (
        log_level_raw
        if isinstance(log_level_raw, LogLevel)
        else LogLevel(str(log_level_raw).lower())
    )
    pretty = bool(kwargs.get("pretty", False))
    return (
        ExecutionPolicy(
            output_format=output_format,
            color=ColorMode.AUTO,
            quiet=bool(kwargs.get("quiet", False)),
            verbose=verbose,
            verbose_level=1 if verbose else 0,
            log_level=log_level,
            pretty=pretty,
            include_runtime=verbose,
            json=output_format is OutputFormat.JSON,
        ),
        output_format,
        output_format,
    )


def test_build_payload_path_empty(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a missing PATH environment variable is detected as unhealthy."""
    monkeypatch.delenv("PATH", raising=False)
    monkeypatch.delenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", raising=False)

    payload = _build_payload(include_runtime=False)
    assert payload.status == "unhealthy"
    assert "Environment PATH is empty" in payload.summary


def test_build_payload_force_unhealthy(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the unhealthy status can be forced via an environment variable."""
    monkeypatch.setenv("PATH", "/usr/bin")
    monkeypatch.setenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", "1")

    payload = _build_payload(include_runtime=False)
    assert payload.status == "unhealthy"
    assert "Forced unhealthy by test environment" in payload.summary


def test_build_payload_combined_issues(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the payload correctly reports multiple combined issues."""
    monkeypatch.delenv("PATH", raising=False)
    monkeypatch.setenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", "1")

    payload = _build_payload(include_runtime=True)
    assert payload.status == "unhealthy"
    assert "Environment PATH is empty" in payload.summary
    assert "Forced unhealthy by test environment" in payload.summary
    assert payload.python
    assert payload.platform


def test_build_payload_all_healthy(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the payload structure for a healthy system with and without runtime info."""
    monkeypatch.setenv("PATH", "/usr/bin")
    monkeypatch.delenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", raising=False)

    payload = _build_payload(include_runtime=False)
    assert payload.status == "healthy"
    assert payload.summary == ["All core checks passed"]
    assert payload.python is None
    assert payload.platform is None

    payload_rt = _build_payload(include_runtime=True)
    assert payload_rt.status == "healthy"
    assert payload_rt.summary == ["All core checks passed"]
    assert isinstance(payload_rt.python, str)
    assert isinstance(payload_rt.platform, str)


def test_build_payload_detects_empty_path_and_forced_unhealthy(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that both empty PATH and forced unhealthy states are correctly detected."""
    monkeypatch.delenv("PATH", raising=False)
    monkeypatch.delenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", raising=False)
    p1 = _build_payload(False)
    assert p1.status == "unhealthy"
    assert any("Environment PATH is empty" in msg for msg in p1.summary)

    monkeypatch.setenv("PATH", "/usr/bin")
    monkeypatch.setenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", "1")
    p2 = _build_payload(False)
    assert p2.status == "unhealthy"
    assert any("Forced unhealthy by test environment" in msg for msg in p2.summary)


def test_doctor_short_circuits_if_subcommand_set() -> None:
    """Test that the doctor command returns early if a subcommand is invoked."""
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = "anything"
    result = doctor(
        ctx,
        quiet=False,
        verbose=False,
        fmt="json",
        pretty=True,
        log_level=LogLevel.INFO,
    )
    assert result is None


def test_doctor_di_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    """Handle DI resolution failure in `doctor`."""
    monkeypatch.setenv("PATH", "/usr/bin")

    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.doctor.resolve_command_config",
        _fake_resolve_command_config,
        raising=False,
    )

    fake_di = MagicMock()
    fake_di.resolve.side_effect = Exception("boom")
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.doctor.DIContainer.current",
        lambda: fake_di,
        raising=False,
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []

    with patch(
        "bijux_cli.cli.commands.diagnostics.doctor.emit_error_and_exit"
    ) as mock_emit:
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            doctor(
                ctx,
                quiet=False,
                verbose=False,
                fmt="json",
                pretty=True,
                log_level=LogLevel.INFO,
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
        "bijux_cli.cli.commands.diagnostics.doctor.resolve_command_config",
        _fake_resolve_command_config,
        raising=False,
    )

    fake_di = MagicMock()
    fake_di.resolve.return_value = None
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.doctor.DIContainer.current",
        lambda: fake_di,
        raising=False,
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []

    with patch("bijux_cli.cli.commands.diagnostics.doctor.new_run_command") as mock_new:
        doctor(
            ctx,
            quiet=True,
            verbose=True,
            fmt="yaml",
            pretty=False,
            log_level=LogLevel.DEBUG,
        )

    mock_new.assert_called_once()
    kw = mock_new.call_args.kwargs
    assert kw["command_name"] == "doctor"
    builder = kw["payload_builder"]
    p0 = builder(False)
    assert p0.status
    assert p0.summary
    p1 = builder(True)
    assert p1.python
    assert p1.platform


@patch("bijux_cli.cli.commands.diagnostics.doctor.emit_error_and_exit")
@patch(
    "bijux_cli.cli.commands.diagnostics.doctor.resolve_command_config",
    autospec=True,
)
def test_doctor_stray_option_calls_emit_and_exits(
    mock_resolve: MagicMock, mock_emit: MagicMock
) -> None:
    """Test that a stray unknown option results in a structured error."""
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_command_config

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = ["-x"]

    with pytest.raises(SystemExit):
        doctor(
            ctx,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

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


@patch("bijux_cli.cli.commands.diagnostics.doctor.emit_error_and_exit")
@patch(
    "bijux_cli.cli.commands.diagnostics.doctor.resolve_command_config",
    autospec=True,
)
def test_doctor_stray_argument_calls_emit_and_exits(
    mock_resolve: MagicMock, mock_emit: MagicMock
) -> None:
    """Test that a stray argument results in a structured error."""
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_command_config

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = ["foo"]

    with pytest.raises(SystemExit):
        doctor(
            ctx,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

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
