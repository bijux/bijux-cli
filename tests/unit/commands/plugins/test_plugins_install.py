# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins install module."""

from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest
from typer.testing import CliRunner

import bijux_cli.cli.plugins.commands.install as install_mod
from bijux_cli.cli.root import app as cli_app


@pytest.fixture
def captured(monkeypatch: pytest.MonkeyPatch) -> dict[str, Any]:
    """Patch out I/O and capture payloads and errors."""
    data: dict[str, Any] = {}

    monkeypatch.setattr(
        install_mod,
        "validate_common_flags",
        lambda fmt, cmd, quiet, **_kwargs: fmt,
    )

    def fake_new_run(*args: Any, **kwargs: Any) -> None:
        cmd = kwargs.get("command_name") or args[0]
        builder = kwargs.get("payload_builder") or args[1]
        data.update(
            {
                "command": cmd,
                "payload": builder(include=True),
                "quiet": kwargs.get("quiet"),
                "fmt": kwargs.get("fmt"),
                "pretty": kwargs.get("pretty"),
                "log_level": kwargs.get("log_level"),
            }
        )

    monkeypatch.setattr(install_mod, "new_run_command", fake_new_run)

    def fake_emit(
        msg: str,
        code: int,
        failure: str,
        **kwargs: Any,
    ) -> None:
        raise RuntimeError({"message": msg, "code": code, "failure": failure})

    monkeypatch.setattr(install_mod, "raise_exit_intent", fake_emit)
    return data


@pytest.fixture
def runner() -> CliRunner:
    """Provide a CliRunner instance."""
    return CliRunner()


def test_local_path_rejected(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that local paths are rejected."""
    plug = tmp_path / "plug"
    plug.mkdir()
    result = runner.invoke(cli_app, ["plugins", "install", str(plug)])
    assert result.exit_code == 1
    assert result.exception is not None
    assert result.exception.args[0]["failure"] == "local_path_not_supported"


def test_invalid_package_name(captured: dict[str, Any], runner: CliRunner) -> None:
    """Test that invalid package names are rejected."""
    result = runner.invoke(cli_app, ["plugins", "install", "bad name"])
    assert result.exit_code == 1
    assert result.exception is not None
    assert result.exception.args[0]["failure"] == "invalid_name"


def test_dry_run(
    captured: dict[str, Any], runner: CliRunner, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test dry-run output without invoking pip."""
    monkeypatch.setattr(
        install_mod,
        "subprocess",
        SimpleNamespace(run=lambda *a, **k: (_ for _ in ()).throw(AssertionError())),
    )
    result = runner.invoke(cli_app, ["plugins", "install", "--dry-run", "goodpkg"])
    assert result.exit_code == 0
    assert captured["payload"]["status"] == "dry-run"
    assert captured["payload"]["package"] == "goodpkg"


def test_pip_failure(
    captured: dict[str, Any], runner: CliRunner, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that pip install failures are reported."""
    proc = SimpleNamespace(returncode=1, stderr="nope", stdout="")
    monkeypatch.setattr(
        install_mod, "subprocess", SimpleNamespace(run=lambda *a, **k: proc)
    )
    result = runner.invoke(cli_app, ["plugins", "install", "goodpkg"])
    assert result.exit_code == 1
    assert result.exception is not None
    assert result.exception.args[0]["failure"] == "pip_install_failed"


def test_metadata_error(
    captured: dict[str, Any], runner: CliRunner, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that metadata errors are surfaced."""
    proc = SimpleNamespace(returncode=0, stderr="", stdout="")
    monkeypatch.setattr(
        install_mod, "subprocess", SimpleNamespace(run=lambda *a, **k: proc)
    )
    monkeypatch.setattr(
        install_mod,
        "discover_plugins",
        lambda: (_ for _ in ()).throw(RuntimeError("bad")),
    )
    result = runner.invoke(cli_app, ["plugins", "install", "goodpkg"])
    assert result.exit_code == 1
    assert result.exception is not None
    assert result.exception.args[0]["failure"] == "metadata_error"


def test_entrypoint_missing(
    captured: dict[str, Any], runner: CliRunner, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that missing entry points are reported."""
    proc = SimpleNamespace(returncode=0, stderr="", stdout="")
    monkeypatch.setattr(
        install_mod, "subprocess", SimpleNamespace(run=lambda *a, **k: proc)
    )
    monkeypatch.setattr(install_mod, "discover_plugins", lambda: [])
    monkeypatch.setattr(install_mod, "plugins_for_package", lambda _: [])
    result = runner.invoke(cli_app, ["plugins", "install", "goodpkg"])
    assert result.exit_code == 1
    assert result.exception is not None
    assert result.exception.args[0]["failure"] == "entrypoint_missing"


def test_successful_install(
    captured: dict[str, Any], runner: CliRunner, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test a successful PyPI install path."""
    proc = SimpleNamespace(returncode=0, stderr="", stdout="")
    monkeypatch.setattr(
        install_mod, "subprocess", SimpleNamespace(run=lambda *a, **k: proc)
    )
    monkeypatch.setattr(install_mod, "discover_plugins", lambda: [])
    monkeypatch.setattr(
        install_mod,
        "plugins_for_package",
        lambda _: [SimpleNamespace(name="smoke")],
    )
    result = runner.invoke(cli_app, ["plugins", "install", "goodpkg"])
    assert result.exit_code == 0
    assert captured["payload"]["status"] == "installed"
    assert captured["payload"]["package"] == "goodpkg"
    assert captured["payload"]["plugins"] == ["smoke"]
