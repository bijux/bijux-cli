# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI root cli module."""

from __future__ import annotations

import subprocess
import sys
from unittest.mock import Mock

import pytest
import typer

from bijux_cli.cli.color import set_color_mode
import bijux_cli.cli.root as cli_mod
from bijux_cli.core.enums import ColorMode
from bijux_cli.core.precedence import default_execution_policy


@pytest.fixture(autouse=True)
def _default_policy(monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensure a default execution policy for CLI output helpers."""
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: default_execution_policy(),
    )
    monkeypatch.setattr(
        cli_mod,
        "current_execution_policy",
        lambda: default_execution_policy(),
    )


def test_collect_names() -> None:
    """Test the _collect_names helper function with lists and mappings."""

    class DummyCommand:
        def __init__(self, name: str | None) -> None:
            self.name = name

    list_container = [DummyCommand("foo"), DummyCommand(None), object()]
    assert cli_mod._collect_names(list_container) == ["foo"]

    map_container = {"a": DummyCommand("bar"), "b": DummyCommand(None), "c": object()}
    assert cli_mod._collect_names(map_container) == ["bar"]


def test_existing_top_level_names() -> None:
    """Test retrieving names of already registered commands and groups."""
    app = typer.Typer()
    app.command("cmd1")(lambda: None)
    app.add_typer(typer.Typer(), name="group1")

    names = cli_mod._existing_top_level_names(app)
    assert names == {"cmd1", "group1"}


def test_log_registered(caplog: pytest.LogCaptureFixture) -> None:
    """Test that registered command and group names are logged."""
    app = typer.Typer()
    app.command("cmd1")(lambda: None)
    app.add_typer(typer.Typer(), name="group1")

    with caplog.at_level("DEBUG"):
        cli_mod._log_registered(app)

    assert "Core commands registered: ['cmd1']" in caplog.text
    assert "Core groups registered: ['group1']" in caplog.text


def test_build_app(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the main build_app factory."""
    monkeypatch.setattr(cli_mod, "register_commands", lambda *a, **k: None)
    monkeypatch.setattr(cli_mod, "register_dynamic_plugins", lambda *a, **k: None)
    monkeypatch.setattr(cli_mod, "register_entrypoint_plugins", lambda *a, **k: None)

    app = cli_mod.build_app()
    assert isinstance(app, typer.Typer)
    assert app.info.help == "Bijux CLI – Lean, plug-in-driven command-line interface."
    assert app.callback is not None


def test_module_level_app_is_built() -> None:
    """Test that the module-level app is an instance of a built app."""
    assert isinstance(cli_mod.app, typer.Typer)


def test_maybe_default_to_repl_invokes_repl_on_no_args(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that the REPL is invoked when no args are provided."""
    monkeypatch.setattr(sys, "argv", ["bijux"])
    mock_call = Mock()
    monkeypatch.setattr(subprocess, "call", mock_call)
    ctx = Mock(spec=typer.Context, invoked_subcommand=None)

    cli_mod.maybe_default_to_repl(ctx)
    mock_call.assert_called_once_with(["bijux", "repl"])


def test_maybe_default_to_repl_shows_help_on_failed_command(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that help is shown for an unresolved command with arguments."""
    set_color_mode(ColorMode.AUTO)
    monkeypatch.setattr(sys, "argv", ["bijux", "badcmd"])
    mock_echo = Mock()
    monkeypatch.setattr(typer, "echo", mock_echo)
    ctx = Mock(spec=typer.Context, invoked_subcommand=None)
    ctx.get_help.return_value = "Usage: ..."

    with pytest.raises(typer.Exit) as exc_info:
        cli_mod.maybe_default_to_repl(ctx)

    assert exc_info.value.exit_code == 2
    mock_echo.assert_called_once_with("Usage: ...", color=None)


def test_maybe_default_to_repl_does_nothing_with_subcommand(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that nothing happens if a subcommand was successfully invoked."""
    mock_call = Mock()
    mock_echo = Mock()
    monkeypatch.setattr(subprocess, "call", mock_call)
    monkeypatch.setattr(typer, "echo", mock_echo)
    ctx = Mock(spec=typer.Context, invoked_subcommand="goodcmd")

    cli_mod.maybe_default_to_repl(ctx)
    mock_call.assert_not_called()
    mock_echo.assert_not_called()
