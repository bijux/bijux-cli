# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI root cli module."""



from __future__ import annotations

import importlib.metadata as md
import subprocess
import sys
from typing import Any
from unittest.mock import MagicMock, Mock

import pytest
import typer

import bijux_cli.cli as cli_mod


def test_collect_names() -> None:
    """Test the _collect_names helper function with lists and mappings."""

    class DummyCommand:
        def __init__(self, name: str | None) -> None:
            self.name = name

    list_container = [DummyCommand("foo"), DummyCommand(None), object()]
    assert cli_mod._collect_names(list_container) == ["foo"]

    map_container = {"a": DummyCommand("bar"), "b": DummyCommand(None), "c": object()}
    assert cli_mod._collect_names(map_container) == ["bar"]


def test_safe_add_typer() -> None:
    """Test that _safe_add_typer adds a sub-app only if the name is new."""
    app = typer.Typer()
    sub = typer.Typer(name="sub")
    seen = {"existing"}

    cli_mod._safe_add_typer(app, sub, "new", seen)
    assert "new" in seen
    assert len(app.registered_groups) == 1

    cli_mod._safe_add_typer(app, sub, "existing", seen)
    assert len(app.registered_groups) == 1


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


def test_build_app(mocker: Any) -> None:
    """Test the main build_app factory."""
    mocker.patch.object(cli_mod, "register_commands")
    mocker.patch.object(cli_mod, "register_dynamic_plugins")
    mocker.patch.object(cli_mod, "register_entrypoint_plugins")

    app = cli_mod.build_app()
    assert isinstance(app, typer.Typer)
    assert app.info.help == "Bijux CLI – Lean, plug-in-driven command-line interface."
    assert app.callback is not None


def test_module_level_app_is_built() -> None:
    """Test that the module-level app is an instance of a built app."""
    assert isinstance(cli_mod.app, typer.Typer)


def test_maybe_default_to_repl_invokes_repl_on_no_args(mocker: Any) -> None:
    """Test that the REPL is invoked when no args are provided."""
    mocker.patch.object(sys, "argv", ["bijux"])
    mock_call = mocker.patch.object(subprocess, "call")
    ctx = Mock(spec=typer.Context, invoked_subcommand=None)

    cli_mod.maybe_default_to_repl(ctx)
    mock_call.assert_called_once_with(["bijux", "repl"])


def test_maybe_default_to_repl_shows_help_on_failed_command(mocker: Any) -> None:
    """Test that help is shown for an unresolved command with arguments."""
    mocker.patch.object(sys, "argv", ["bijux", "badcmd"])
    mock_echo = mocker.patch("typer.echo")
    ctx = Mock(spec=typer.Context, invoked_subcommand=None)
    ctx.get_help.return_value = "Usage: ..."

    with pytest.raises(typer.Exit) as exc_info:
        cli_mod.maybe_default_to_repl(ctx)

    assert exc_info.value.exit_code == 2
    mock_echo.assert_called_once_with("Usage: ...")


def test_maybe_default_to_repl_does_nothing_with_subcommand(mocker: Any) -> None:
    """Test that nothing happens if a subcommand was successfully invoked."""
    mock_call = mocker.patch.object(subprocess, "call")
    mock_echo = mocker.patch("typer.echo")
    ctx = Mock(spec=typer.Context, invoked_subcommand="goodcmd")

    cli_mod.maybe_default_to_repl(ctx)
    mock_call.assert_not_called()
    mock_echo.assert_not_called()


def test_iter_entry_points_modern(mocker: Any) -> None:
    """Test the entry point iterator uses the modern API."""
    mock_entry_points = mocker.patch.object(md, "entry_points")
    mock_entry_points.return_value = [Mock()]

    result = list(cli_mod._iter_entry_points("my.group"))

    assert len(result) == 1
    mock_entry_points.assert_called_once_with(group="my.group")


def create_mock_ep(name: str, group: str, load_value: Any) -> MagicMock:
    """Helper to create a mock EntryPoint."""
    ep = MagicMock(spec=md.EntryPoint)
    ep.name = name
    ep.group = group
    ep.load.return_value = load_value
    return ep


def test_register_entrypoint_plugins_all_cases(
    mocker: Any, caplog: pytest.LogCaptureFixture
) -> None:
    """Test all plugin registration paths, including failures."""
    app = typer.Typer()

    ep1 = create_mock_ep("modern", "bijux.commands", typer.Typer())
    ep2 = create_mock_ep("not_typer", "bijux.commands", object())
    ep3 = create_mock_ep("load_fail", "bijux.commands", None)
    ep3.load.side_effect = ImportError("Load failed")

    ep4 = create_mock_ep("legacy_typer", "bijux_cli.plugins", typer.Typer())
    ep5_sub = typer.Typer()
    ep5_plugin = Mock(registered_groups={"rg_plugin": ep5_sub}, app=None, register=None)
    ep5 = create_mock_ep("legacy_rg", "bijux_cli.plugins", lambda: ep5_plugin)
    ep6_hook = Mock()
    ep6_plugin = Mock(registered_groups=None, app=None, register=ep6_hook)
    ep6 = create_mock_ep("legacy_hook", "bijux_cli.plugins", lambda: ep6_plugin)
    ep7_hook_fail = Mock(side_effect=RuntimeError("Hook failed"))
    ep7_plugin = Mock(registered_groups=None, app=None, register=ep7_hook_fail)
    ep7 = create_mock_ep("legacy_hook_fail", "bijux_cli.plugins", lambda: ep7_plugin)
    ep8_app = typer.Typer()
    ep8_plugin = Mock(registered_groups=None, app=ep8_app, register=None)
    ep8 = create_mock_ep("legacy_app", "bijux_cli.plugins", lambda: ep8_plugin)
    ep9_inst_fail = Mock(side_effect=TypeError("Cannot instantiate"))
    ep9 = create_mock_ep("inst_fail", "bijux_cli.plugins", ep9_inst_fail)
    ep10_top_level_fail = create_mock_ep("top_fail", "bijux_cli.plugins", None)
    ep10_top_level_fail.load.side_effect = SystemError("Top-level fail")

    mocker.patch.object(
        cli_mod,
        "_iter_entry_points",
        lambda group: {
            "bijux.commands": [ep1, ep2, ep3],
            "bijux_cli.plugins": [ep4, ep5, ep6, ep7, ep8, ep9, ep10_top_level_fail],
        }.get(group, []),
    )

    with caplog.at_level("DEBUG"):
        cli_mod.register_entrypoint_plugins(app)

    assert len(app.registered_groups) == 4
    registered_names = {g.name for g in app.registered_groups}
    assert registered_names == {"modern", "legacy_typer", "rg_plugin", "legacy_app"}

    ep6_hook.assert_called_once_with(app)

    assert "is not a Typer app" in caplog.text
    assert "Failed to load entry point load_fail" in caplog.text
    assert "Failed to instantiate plugin inst_fail" in caplog.text
    assert "register(app) failed: Hook failed" in caplog.text
    assert "Failed to load plugin entry point top_fail" in caplog.text
