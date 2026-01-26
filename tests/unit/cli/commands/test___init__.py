# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the commands' module init."""

from __future__ import annotations

from pathlib import Path
from typing import Any, cast

import pytest
from typer import Typer

from bijux_cli.cli.commands import (
    _CORE_COMMANDS,
    _REGISTERED_COMMANDS,
    list_registered_command_names,
    register_commands,
    register_dynamic_plugins,
)


def test_register_commands_adds_all_core_commands(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that all core commands are registered with the main Typer app."""
    app = Typer()
    added: list[tuple[str, bool]] = []

    def fake_add_typer(cmd: Typer, name: str, invoke_without_command: bool) -> None:
        added.append((name, invoke_without_command))

    monkeypatch.setattr(app, "add_typer", fake_add_typer)

    names = register_commands(app)
    expected = sorted(_CORE_COMMANDS.keys())
    assert names == expected

    assert sorted(n for n, _ in added) == expected
    assert all(inv for _, inv in added)


def make_fake_ep(name: str, app_obj: Any = None, exc: Exception | None = None) -> Any:
    """Create a fake entry point object for testing."""

    class EP:
        """A mock entry point class."""

        def __init__(self) -> None:
            """Initialize the mock entry point."""
            self.name = name

        def load(self) -> Any:
            """Load the mock entry point, returning an object or raising an exception."""
            if exc:
                raise exc
            return app_obj

    return EP()


class DummyTyper(Typer):
    """A dummy Typer subclass for type checking tests."""


def test_list_registered_command_names_includes_cores_and_plugins() -> None:
    """Test that the list of registered commands includes all expected commands."""
    all_names = list_registered_command_names()
    for core in _CORE_COMMANDS:
        assert core in all_names


def test_list_registered_command_names_collects_dynamic(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that dynamically added plugins are included in the registered command list."""
    name = "zz_test_plugin"
    _REGISTERED_COMMANDS.add(name)

    all_names = list_registered_command_names()
    assert name in all_names


def test_register_dynamic_plugins_via_entry_points(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test the registration of dynamic plugins via package entry points."""
    root = Typer()
    added: list[str] = []
    monkeypatch.setattr(root, "add_typer", lambda app, name: added.append(name))

    from bijux_cli.plugins.metadata import PluginMetadata

    class FakeEntryPoint:
        def __init__(self, name: str, app_obj: Any) -> None:
            self.name = name
            self._app_obj = app_obj

        def load(self) -> Any:
            return self._app_obj

    good = PluginMetadata(
        name="good_ep",
        version="0.1.0",
        enabled=True,
        source="entrypoint",
        requires_cli=">=0.1.0",
        entrypoint=cast(Any, FakeEntryPoint("good_ep", DummyTyper())),
    )
    bad = PluginMetadata(
        name="bad_ep",
        version="0.1.0",
        enabled=True,
        source="entrypoint",
        requires_cli=">=0.1.0",
        entrypoint=cast(Any, FakeEntryPoint("bad_ep", object())),
    )

    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.discover_plugins", lambda: [good, bad]
    )
    monkeypatch.setattr(
        "bijux_cli.plugins.loader.activate_plugin",
        lambda meta: DummyTyper()
        if meta.name == "good_ep"
        else (_ for _ in ()).throw(RuntimeError("fail_load")),
    )

    before = set(list_registered_command_names())
    register_dynamic_plugins(root)
    after = set(list_registered_command_names())

    assert "good_ep" in added
    assert "bad_ep" not in added
    assert "good_ep" in after - before
    assert "bad_ep" not in after - before


def test_dynamic_plugins_entry_point_loading_fails_entire_metadata(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a failure in loading entry points does not crash the system."""
    root = Typer()
    monkeypatch.setattr(root, "add_typer", lambda *a, **k: None)
    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.discover_plugins",
        lambda: (_ for _ in ()).throw(RuntimeError("broken")),
    )

    before = set(list_registered_command_names())
    register_dynamic_plugins(root)
    after = set(list_registered_command_names())
    assert before == after


def test_dynamic_plugins_discovery_bails_on_getdir_exception(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that an exception during plugin directory discovery is handled."""
    root = Typer()
    monkeypatch.setattr(root, "add_typer", lambda *a, **k: None)
    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.discover_plugins",
        lambda: (_ for _ in ()).throw(ValueError("no dirs")),
    )

    before = set(list_registered_command_names())
    register_dynamic_plugins(root)
    after = set(list_registered_command_names())
    assert before == after


def test_register_dynamic_plugins_rejects_name_collision(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    root = Typer()
    monkeypatch.setattr(root, "add_typer", lambda *a, **k: None)

    class _Meta:
        name = "dup"

    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.discover_plugins", lambda: [_Meta()]
    )
    monkeypatch.setattr("bijux_cli.plugins.loader.activate_plugin", lambda _m: Typer())
    monkeypatch.setattr("bijux_cli.cli.commands._REGISTERED_COMMANDS", {"dup"})

    with pytest.raises(RuntimeError, match="Plugin name collision"):
        register_dynamic_plugins(root)
