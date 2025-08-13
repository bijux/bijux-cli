# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins list module."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import pytest
from typer.testing import CliRunner

from bijux_cli.cli import app as cli_app
import bijux_cli.commands.plugins.list as list_mod


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

    def fake_handle(
        command: str, quiet: bool, verbose: bool, fmt: str, pretty: bool, debug: bool
    ) -> None:
        calls["handle"] = (command, quiet, verbose, fmt, pretty, debug)

    monkeypatch.setattr(list_mod, "handle_list_plugins", fake_handle)

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
    assert caps["handle"] == ("plugins list", False, False, "json", True, False)


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
            "--debug",
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
        True,
    )
    assert caps["handle"] == ("plugins list", True, True, "yaml", False, True)


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
