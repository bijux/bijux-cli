# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the sleep command."""

from __future__ import annotations

import json
from types import SimpleNamespace
from typing import Any

import pytest
from typer.testing import CliRunner

from bijux_cli.commands.sleep import (
    _build_payload,  # pyright: ignore[reportPrivateUsage]
    sleep_app,
)

runner: CliRunner = CliRunner()


def test_build_payload_no_runtime() -> None:
    """Builds payload without runtime fields."""
    payload = _build_payload(include_runtime=False, slept=1.25)  # pyright: ignore[reportPrivateUsage]
    assert payload == {"slept": 1.25}


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
        "bijux_cli.commands.sleep.ascii_safe", fake_ascii_safe, raising=True
    )
    monkeypatch.setattr("platform.python_version", fake_pyver, raising=True)
    monkeypatch.setattr("platform.platform", fake_platform, raising=True)

    payload = _build_payload(include_runtime=True, slept=0.5)  # pyright: ignore[reportPrivateUsage]
    assert payload["slept"] == 0.5
    assert payload["python"] == "SAFE(python_version)"
    assert payload["platform"] == "SAFE(platform)"
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

    fake_container = SimpleNamespace(
        resolve=lambda _proto: FakeCfg()  # pyright: ignore[reportUnknownLambdaType]
    )
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
    payload = json.loads(result.output.strip())
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
    payload = json.loads(result.output.strip())
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
    payload = json.loads(result.output.strip())
    assert payload["failure"] == "timeout"
    assert payload["code"] == 2


def test_sleep_success(monkeypatch: pytest.MonkeyPatch) -> None:
    """Succeeds and returns payload with runtime when verbose."""
    _install_fake_container(monkeypatch, get_returns="10")

    def _sleep(_s: float) -> None:
        return None

    monkeypatch.setattr("time.sleep", _sleep, raising=True)

    result = runner.invoke(
        sleep_app,
        ["--seconds", "0.2", "--format", "json", "--verbose", "--pretty"],
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
