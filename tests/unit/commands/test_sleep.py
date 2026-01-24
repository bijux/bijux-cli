# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the sleep command."""

from __future__ import annotations

import json
from types import SimpleNamespace
from typing import Any, cast

import pytest
from typer.testing import CliRunner

from bijux_cli.cli.commands.sleep import (
    _build_payload,
    sleep_app,
)
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import ExecutionPolicy
from bijux_cli.infra.contracts import Emitter, Serializer
from bijux_cli.services.config.contracts import ConfigProtocol
from bijux_cli.services.contracts import TelemetryProtocol

runner: CliRunner = CliRunner()


def _load_payload(result: Any) -> dict[str, Any]:
    """Load a JSON payload from stdout or stderr."""
    raw = (result.output or result.stderr or "").strip()
    return cast(dict[str, Any], json.loads(raw))


def test_build_payload_no_runtime() -> None:
    """Builds payload without runtime fields."""
    payload = _build_payload(include_runtime=False, slept=1.25)
    assert payload.slept == 1.25
    assert payload.python is None
    assert payload.platform is None


def test_build_payload_with_runtime(monkeypatch: pytest.MonkeyPatch) -> None:
    """Builds payload with runtime fields."""
    calls: list[tuple[str, str]] = []

    def fake_ascii_safe(val: Any, field: str) -> str:
        calls.append((str(val), field))
        return f"SAFE({field})"

    def fake_pyver() -> str:
        return "3.11.9"

    def fake_platform() -> str:
        return "TestOS-1.0"

    monkeypatch.setattr(
        "bijux_cli.cli.commands.sleep.ascii_safe", fake_ascii_safe, raising=True
    )
    monkeypatch.setattr("platform.python_version", fake_pyver, raising=True)
    monkeypatch.setattr("platform.platform", fake_platform, raising=True)

    payload = _build_payload(include_runtime=True, slept=0.5)
    assert payload.slept == 0.5
    assert payload.python == "SAFE(python_version)"
    assert payload.platform == "SAFE(platform)"
    assert {field for _, field in calls} == {"python_version", "platform"}


def _install_fake_container(
    monkeypatch: pytest.MonkeyPatch,
    *,
    get_returns: str | None = None,
    get_raises: Exception | None = None,
) -> None:
    """Install a fake DI container that returns a config object with .get()."""

    class FakeCfg:
        """Fake config with .get()."""

        def get(self, key: str, default: str) -> str:
            if get_raises:
                raise get_raises
            return get_returns if get_returns is not None else default

    class _TestSerializer:
        def dumps(self, payload: Any, fmt: OutputFormat, pretty: bool = False) -> str:
            from bijux_cli.cli.core.command import normalize_payload

            return json.dumps(normalize_payload(payload))

    class _TestEmitter:
        pass

    class _TestTelemetry:
        pass

    def _resolve(proto: Any) -> Any:
        if proto is ConfigProtocol:
            return FakeCfg()
        if proto is Serializer:
            return _TestSerializer()
        if proto is Emitter:
            return _TestEmitter()
        if proto is TelemetryProtocol:
            return _TestTelemetry()
        raise KeyError(f"Unexpected resolve: {proto}")

    fake_container = SimpleNamespace(resolve=_resolve)
    monkeypatch.setattr(
        "bijux_cli.core.di.DIContainer.current",
        staticmethod(lambda: fake_container),
        raising=True,
    )


def test_sleep_negative_seconds(monkeypatch: pytest.MonkeyPatch) -> None:
    """Errors on negative seconds."""

    def _sleep(_s: float) -> None:
        return None

    monkeypatch.setattr("time.sleep", _sleep, raising=True)

    result = runner.invoke(sleep_app, ["--seconds", "-1", "--format", "json"])
    assert result.exit_code != 0
    payload = _load_payload(result)
    assert payload["failure"] == "negative"
    assert payload["code"] == 2


def test_sleep_config_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Errors when configuration fetch fails."""
    _install_fake_container(monkeypatch, get_raises=Exception("boom"))

    def _sleep(_s: float) -> None:
        return None

    monkeypatch.setattr("time.sleep", _sleep, raising=True)

    result = runner.invoke(sleep_app, ["--seconds", "0", "--format", "json"])
    assert result.exit_code != 0
    payload = _load_payload(result)
    assert payload["failure"] == "config"
    assert "Failed to read timeout" in payload["error"]


def test_sleep_timeout_exceeded(monkeypatch: pytest.MonkeyPatch) -> None:
    """Errors when requested seconds exceed configured timeout."""
    _install_fake_container(monkeypatch, get_returns="0.01")

    def _sleep(_s: float) -> None:
        return None

    monkeypatch.setattr("time.sleep", _sleep, raising=True)

    result = runner.invoke(sleep_app, ["--seconds", "1.0", "--format", "json"])
    assert result.exit_code != 0
    payload = _load_payload(result)
    assert payload["failure"] == "timeout"
    assert payload["code"] == 2


def test_sleep_success(monkeypatch: pytest.MonkeyPatch) -> None:
    """Succeeds and returns payload with runtime when debug."""
    _install_fake_container(monkeypatch, get_returns="10")
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=True,
        ),
    )

    def _sleep(_s: float) -> None:
        return None

    monkeypatch.setattr("time.sleep", _sleep, raising=True)

    result = runner.invoke(
        sleep_app,
        ["--seconds", "0.2", "--format", "json", "--log-level", "debug", "--pretty"],
    )
    assert result.exit_code == 0

    text = result.output
    end = text.find("}\n")
    if end != -1:
        text = text[: end + 2]
    payload = json.loads(text)
    assert pytest.approx(payload["slept"], rel=1e-6) == 0.2
    assert "python" in payload
    assert "platform" in payload
