# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins uninstall module."""

# mypy: disable-error-code="union-attr"
# pyright: reportOptionalMemberAccess=false
from __future__ import annotations

from pathlib import Path
from typing import Any

import pytest
from typer.testing import CliRunner

import bijux_cli.cli as cli
from bijux_cli.cli import app as cli_app
import bijux_cli.commands.plugins.uninstall as uninstall_mod


@pytest.fixture
def captured(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> dict[str, Any]:
    """Stub out I/O and capture new_run_command payloads and errors."""
    data: dict[str, Any] = {}

    monkeypatch.setattr(uninstall_mod, "get_plugins_dir", lambda: tmp_path / "plugins")
    monkeypatch.setattr(
        uninstall_mod, "refuse_on_symlink", lambda *args, **kwargs: None
    )
    monkeypatch.setattr(
        uninstall_mod, "validate_common_flags", lambda fmt, cmd, quiet: fmt
    )

    def fake_new_run(
        command_name: str,
        payload_builder: Any,
        quiet: bool,
        verbose: bool,
        fmt: str,
        pretty: bool,
        debug: bool,
    ) -> None:
        data["payload"] = payload_builder(include=True)

    monkeypatch.setattr(uninstall_mod, "new_run_command", fake_new_run)

    def fake_emit_error_and_exit(
        message: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
    ) -> None:
        raise RuntimeError({"message": message, "code": code, "failure": failure})

    monkeypatch.setattr(uninstall_mod, "emit_error_and_exit", fake_emit_error_and_exit)
    return data


@pytest.fixture
def runner() -> CliRunner:
    """Provide a CliRunner instance."""
    return CliRunner()


@pytest.fixture(autouse=True)
def stub_plugins_dir_and_capture(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> dict[str, Any]:
    """Stub the plugins directory and capture the payload from new_run_command."""
    plugins_dir = tmp_path / "plugins"
    plugins_dir.mkdir()
    data: dict[str, Any] = {}

    monkeypatch.setattr(uninstall_mod, "get_plugins_dir", lambda: plugins_dir)

    def fake_new_run(
        command_name: str,
        payload_builder: Any,
        quiet: bool,
        verbose: bool,
        fmt: str,
        pretty: bool,
        debug: bool,
    ) -> None:
        data.update(payload_builder(include=True))

    monkeypatch.setattr(uninstall_mod, "new_run_command", fake_new_run)
    return {"plugins_dir": plugins_dir, "captured": data}


def test_list_failed(captured: dict[str, Any], runner: CliRunner) -> None:
    """Test that an empty plugins directory results in a 'not_installed' error."""
    result = runner.invoke(cli_app, ["plugins", "uninstall", "foo"])
    assert result.exit_code == 1
    err = result.exception.args[0]
    assert err["failure"] == "not_installed"


def test_not_installed(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that attempting to uninstall a non-existent plugin fails."""
    result = runner.invoke(cli_app, ["plugins", "uninstall", "doesntexist"])
    assert result.exit_code == 1
    err = result.exception.args[0]
    assert err["failure"] == "not_installed"


def test_symlink_path(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that attempting to uninstall a symlinked plugin fails."""
    plugins_dir = tmp_path / "plugins"
    real = tmp_path / "real_plugin"
    real.mkdir()
    (real / "plugin.py").write_text("#")
    (plugins_dir / "qux").symlink_to(real, target_is_directory=True)

    result = runner.invoke(cli_app, ["plugins", "uninstall", "qux"])
    assert result.exit_code == 1
    err = result.exception.args[0]
    assert err["failure"] == "symlink_path"


def test_permission_denied(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a PermissionError during uninstallation is handled."""
    plugins_dir = tmp_path / "plugins"
    (plugins_dir / "corge").mkdir(parents=True)

    monkeypatch.setattr(
        uninstall_mod.shutil,  # type: ignore[attr-defined]
        "rmtree",
        lambda p: (_ for _ in ()).throw(PermissionError()),
    )

    result = runner.invoke(cli_app, ["plugins", "uninstall", "corge"])
    assert result.exit_code == 1
    err = result.exception.args[0]
    assert err["failure"] == "permission_denied"


def test_remove_failed(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a generic error during uninstallation is handled."""
    plugins_dir = tmp_path / "plugins"
    (plugins_dir / "grault").mkdir(parents=True)

    monkeypatch.setattr(
        uninstall_mod.shutil,  # type: ignore[attr-defined]
        "rmtree",
        lambda p: (_ for _ in ()).throw(RuntimeError("boom")),
    )

    result = runner.invoke(cli_app, ["plugins", "uninstall", "grault"])
    assert result.exit_code == 1
    err = result.exception.args[0]
    assert err["failure"] == "remove_failed"


def test_successful_uninstall(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test the successful uninstallation of a plugin."""
    plugins_dir = tmp_path / "plugins"
    target = plugins_dir / "garply"
    (target / "dummy.txt").parent.mkdir(parents=True, exist_ok=True)
    (target / "dummy.txt").write_text("data")

    result = runner.invoke(cli_app, ["plugins", "uninstall", "garply"])
    assert result.exit_code == 0

    assert not target.exists()
    payload = captured["payload"]
    assert payload["status"] == "uninstalled"
    assert payload["plugin"] == "garply"


def test_not_dir_to_not_installed(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that attempting to uninstall a file (not a directory) fails."""
    plugins_dir = tmp_path / "plugins"
    (plugins_dir / "bar").write_text("not a dir")

    result = runner.invoke(cli_app, ["plugins", "uninstall", "bar"])
    assert result.exit_code == 1
    err = result.exception.args[0]
    assert err["failure"] == "not_installed"


def test_plugin_disappears_during_lock(
    stub_plugins_dir_and_capture: dict[str, Any], runner: CliRunner
) -> None:
    """Test that the uninstall succeeds if the plugin directory vanishes after locking."""
    plugins_dir = stub_plugins_dir_and_capture["plugins_dir"]
    captured = stub_plugins_dir_and_capture["captured"]

    plugin_dir = plugins_dir / "ghost"
    plugin_dir.mkdir()

    orig_exists = Path.exists

    def fake_exists(self: Path) -> bool:
        if self == plugin_dir:
            return False
        return orig_exists(self)

    monkeypatch = pytest.MonkeyPatch()
    monkeypatch.setattr(Path, "exists", fake_exists)

    result = runner.invoke(cli.app, ["plugins", "uninstall", "ghost"])
    assert result.exit_code == 0

    assert captured == {"status": "uninstalled", "plugin": "ghost"}
    monkeypatch.undo()


def test_not_dir_branch(
    tmp_path: Path,
    runner: CliRunner,
    captured: dict[str, Any],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test the specific error branch for when a path is not a directory."""

    class FakeP:
        def __init__(self, name: str) -> None:
            self.name = name
            self._calls = 0

        def is_dir(self) -> bool:
            self._calls += 1
            return self._calls == 1

        def exists(self) -> bool:
            return True

        def is_symlink(self) -> bool:
            return False

    fake_p = FakeP("qux")

    class FakePluginsDir:
        def __init__(self, real: Path, items: list[Any]) -> None:
            self._real = real
            self._items = items

        def iterdir(self) -> Any:
            return iter(self._items)

        def __truediv__(self, other: str) -> Path:
            return self._real / other

        def is_symlink(self) -> bool:
            return False

    fake_plugins = FakePluginsDir(tmp_path / "plugins", [fake_p])
    monkeypatch.setattr(uninstall_mod, "get_plugins_dir", lambda: fake_plugins)

    result = runner.invoke(cli_app, ["plugins", "uninstall", "qux"])
    assert result.exit_code == 1
    err = result.exception.args[0]
    assert err["failure"] == "not_dir"
    assert "is not a directory" in err["message"]


def test_list_failed_exception(
    monkeypatch: pytest.MonkeyPatch, runner: CliRunner, captured: dict[str, Any]
) -> None:
    """Test that an error during plugin listing is handled."""

    class BrokenDir:
        def iterdir(self) -> None:
            raise RuntimeError("boom")

        def __truediv__(self, other: str) -> Path:
            return Path("/does/not/matter") / other

        def is_symlink(self) -> bool:
            return False

    monkeypatch.setattr(uninstall_mod, "get_plugins_dir", BrokenDir)

    result = runner.invoke(cli_app, ["plugins", "uninstall", "anything"])
    assert result.exit_code == 1

    err = result.exception.args[0]
    assert err["failure"] == "list_failed"
    assert "Could not list plugins dir" in err["message"]
