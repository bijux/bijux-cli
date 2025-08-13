# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the commands' module init."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

import importlib
import importlib.metadata
import importlib.util
from pathlib import Path
import types
from typing import Any

import pytest
from typer import Typer

from bijux_cli.commands import (
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


def test_local_plugin_missing_loader(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test that a local plugin with a missing loader is skipped silently."""
    root = Typer()
    added: list[str] = []
    monkeypatch.setattr(root, "add_typer", lambda app, name: added.append(name))

    base = tmp_path / "plugins"
    base.mkdir()
    p = base / "no_loader"
    p.mkdir()
    (p / "plugin.py").write_text("from typer import Typer\napp = Typer()\n")

    import bijux_cli.services.plugins as serv

    monkeypatch.setattr(serv, "get_plugins_dir", lambda: base)

    real_spec = importlib.util.spec_from_file_location

    def fake_spec(name: str, path: str) -> Any:
        return types.SimpleNamespace(loader=None)

    monkeypatch.setattr(importlib.util, "spec_from_file_location", fake_spec)

    register_dynamic_plugins(root)
    assert not added
    monkeypatch.setattr(importlib.util, "spec_from_file_location", real_spec)


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

    fake_eps = [
        make_fake_ep("good_ep", app_obj=DummyTyper()),
        make_fake_ep("bad_ep", exc=RuntimeError("fail_load")),
    ]

    class FakeEPs:
        def select(self, *, group: str) -> list[Any]:
            assert group == "bijux_cli.plugins"
            return fake_eps

    monkeypatch.setattr(importlib.metadata, "entry_points", FakeEPs)

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
        importlib.metadata,
        "entry_points",
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
        importlib.metadata,
        "entry_points",
        lambda: types.SimpleNamespace(select=lambda **kw: []),
    )

    import bijux_cli.services.plugins as serv

    monkeypatch.setattr(
        serv, "get_plugins_dir", lambda: (_ for _ in ()).throw(ValueError("no dirs"))
    )

    before = set(list_registered_command_names())
    register_dynamic_plugins(root)
    after = set(list_registered_command_names())
    assert before == after


def test_register_dynamic_plugins_local_dir(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test the registration of dynamic plugins from a local directory."""
    root = Typer()
    added: list[str] = []
    monkeypatch.setattr(root, "add_typer", lambda app, name: added.append(name))

    base = tmp_path / "plugins"
    base.mkdir()
    (base / "plug1").mkdir()
    (base / "plug1" / "plugin.py").write_text(
        "from typer import Typer\ndef cli(): return Typer()"
    )
    (base / "plug2").mkdir()
    (base / "plug2" / "plugin.py").write_text("from typer import Typer\napp = Typer()")
    (base / "plug3").mkdir()
    (base / "plug3" / "plugin.py").write_text("x = 5")
    (base / "plug4").mkdir()

    import bijux_cli.services.plugins as serv

    monkeypatch.setattr(serv, "get_plugins_dir", lambda: base)

    register_dynamic_plugins(root)

    assert set(added) == {"plug1", "plug2"}


def test_local_plugin_cli_returns_non_typer(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that a local plugin's 'cli' function returning a non-Typer object is skipped."""
    root = Typer()
    added: list[str] = []
    monkeypatch.setattr(root, "add_typer", lambda app, name: added.append(name))

    base = tmp_path / "plugins"
    base.mkdir()
    p = base / "bad_app"
    p.mkdir()
    (p / "plugin.py").write_text("def cli(): return 12345")

    import bijux_cli.services.plugins as serv

    monkeypatch.setattr(serv, "get_plugins_dir", lambda: base)

    register_dynamic_plugins(root)

    assert not added


def test_local_plugin_module_load_exception(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that an exception during a local plugin's module import is handled."""
    root = Typer()
    added: list[str] = []
    monkeypatch.setattr(root, "add_typer", lambda app, name: added.append(name))

    base = tmp_path / "plugins"
    base.mkdir()
    p = base / "raiser"
    p.mkdir()
    (p / "plugin.py").write_text("raise ValueError('import-fail')")

    import bijux_cli.services.plugins as serv

    monkeypatch.setattr(serv, "get_plugins_dir", lambda: base)

    register_dynamic_plugins(root)

    assert not added


def test_entry_points_loading_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    """Entry point failures shouldn't change the registered command names."""
    before = set(list_registered_command_names())

    root = Typer()
    monkeypatch.setattr(
        importlib.metadata,
        "entry_points",
        lambda: (_ for _ in ()).throw(RuntimeError("ep-boom")),
    )

    register_dynamic_plugins(root)

    after = set(list_registered_command_names())
    assert after == before
    assert "ep-boom" not in after
