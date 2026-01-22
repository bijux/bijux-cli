# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins list module."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import pytest
from typer.testing import CliRunner

import bijux_cli.cli.commands.plugins.list as list_mod
from bijux_cli.cli.root import app as cli_app


@pytest.fixture
def caps(monkeypatch: pytest.MonkeyPatch) -> dict[str, Any]:
    """Provide a dictionary to capture calls to mocked functions."""
    calls: dict[str, Any] = {}

    fake_dir = Path("/fake/plugins")
    monkeypatch.setattr(list_mod, "get_plugins_dir", lambda: fake_dir)
    calls["plugins_dir"] = fake_dir

    def fake_validate(fmt: str, cmd: str, quiet: bool) -> str:
        calls["validate"] = (fmt, cmd, quiet)
        return fmt.lower()

    monkeypatch.setattr(list_mod, "validate_common_flags", fake_validate)

    def fake_refuse(
        dir_: Path, command: str, fmt: str, quiet: bool, verbose: bool, debug: bool
    ) -> None:
        calls["refuse"] = (dir_, command, fmt, quiet, verbose, debug)

    monkeypatch.setattr(list_mod, "refuse_on_symlink", fake_refuse)

    def fake_list() -> list[dict[str, Any]]:
        return [{"name": "p1"}]

    monkeypatch.setattr(list_mod, "list_installed_plugins", fake_list)

    def fake_run(**kwargs: Any) -> None:
        calls["run"] = kwargs

    monkeypatch.setattr(list_mod, "new_run_command", fake_run)

    return calls


@pytest.fixture
def runner() -> CliRunner:
    """Provide a CliRunner instance."""
    return CliRunner()


def test_default_list(caps: dict[str, Any], runner: CliRunner) -> None:
    """Test the 'plugins list' command with default flags."""
    result = runner.invoke(cli_app, ["plugins", "list"])
    assert result.exit_code == 0

    assert caps["validate"] == ("json", "plugins list", False)
    assert caps["refuse"] == (
        caps["plugins_dir"],
        "plugins list",
        "json",
        False,
        False,
        False,
    )
    assert caps["run"]["command_name"] == "plugins list"
    assert caps["run"]["quiet"] is False
    assert caps["run"]["verbose"] is False
    assert caps["run"]["fmt"] == "json"
    assert caps["run"]["pretty"] is True
    assert caps["run"]["log_level"] == "info"


def test_all_flags(caps: dict[str, Any], runner: CliRunner) -> None:
    """Test the 'plugins list' command with all flags specified."""
    result = runner.invoke(
        cli_app,
        [
            "plugins",
            "list",
            "--quiet",
            "--verbose",
            "--format",
            "yaml",
            "--no-pretty",
            "--log-level",
            "debug",
        ],
    )
    assert result.exit_code == 0

    assert caps["validate"] == ("yaml", "plugins list", True)
    assert caps["refuse"] == (
        caps["plugins_dir"],
        "plugins list",
        "yaml",
        True,
        True,
        False,
    )
    assert caps["run"]["command_name"] == "plugins list"
    assert caps["run"]["quiet"] is True
    assert caps["run"]["verbose"] is True
    assert caps["run"]["fmt"] == "yaml"
    assert caps["run"]["pretty"] is False
    assert caps["run"]["log_level"] == "error"


def test_validate_error(monkeypatch: pytest.MonkeyPatch, runner: CliRunner) -> None:
    """Test that a SystemExit from flag validation is propagated."""
    monkeypatch.setattr(
        list_mod,
        "validate_common_flags",
        lambda f, c, q: (_ for _ in ()).throw(SystemExit(2)),
    )
    result = runner.invoke(cli_app, ["plugins", "list"])
    assert result.exit_code == 2


def test_refuse_error(monkeypatch: pytest.MonkeyPatch, runner: CliRunner) -> None:
    """Test that a SystemExit from the symlink check is propagated."""
    monkeypatch.setattr(list_mod, "validate_common_flags", lambda f, c, q: f)
    monkeypatch.setattr(list_mod, "get_plugins_dir", lambda: Path("/x"))
    monkeypatch.setattr(
        list_mod,
        "refuse_on_symlink",
        lambda *a, **k: (_ for _ in ()).throw(SystemExit(3)),
    )

    result = runner.invoke(cli_app, ["plugins", "list"])
    assert result.exit_code == 3
