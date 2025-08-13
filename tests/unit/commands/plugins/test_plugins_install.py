# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins install module."""

# mypy: disable-error-code="union-attr"
# pyright: reportPrivateImportUsage=false
# pyright: reportOptionalMemberAccess=false

from __future__ import annotations

import errno
import importlib
from pathlib import Path
from typing import Any

import pytest
from typer.testing import CliRunner

from bijux_cli.cli import app as cli_app
import bijux_cli.commands.plugins.install as install_mod


@pytest.fixture
def captured(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> dict[str, Any]:
    """Patch out all I/O and capture final payloads and errors."""
    data: dict[str, Any] = {}
    monkeypatch.setattr(install_mod, "get_plugins_dir", lambda: tmp_path / "plugins")
    monkeypatch.setattr(install_mod, "refuse_on_symlink", lambda *args, **kwargs: None)
    monkeypatch.setattr(
        install_mod, "validate_common_flags", lambda fmt, cmd, quiet: fmt
    )

    def fake_new_run(*args: Any, **kwargs: Any) -> None:
        cmd = kwargs.get("command_name") or args[0]
        builder = kwargs.get("payload_builder") or args[1]
        data.update(
            {
                "command": cmd,
                "payload": builder(include=True),
                "quiet": kwargs.get("quiet"),
                "verbose": kwargs.get("verbose"),
                "fmt": kwargs.get("fmt"),
                "pretty": kwargs.get("pretty"),
                "debug": kwargs.get("debug"),
            }
        )

    monkeypatch.setattr(install_mod, "new_run_command", fake_new_run)

    def fake_emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
    ) -> None:
        raise RuntimeError({"message": msg, "code": code, "failure": failure})

    monkeypatch.setattr(install_mod, "emit_error_and_exit", fake_emit)
    version_mod = importlib.import_module("bijux_cli.__version__")
    monkeypatch.setattr(version_mod, "version", "1.0.0")
    monkeypatch.setattr(install_mod, "parse_required_cli_version", lambda path: None)
    return data


@pytest.fixture
def runner() -> CliRunner:
    """Provide a CliRunner instance."""
    return CliRunner()


def test_source_not_found(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that a non-existent source path results in an error."""
    result = runner.invoke(cli_app, ["plugins", "install", str(tmp_path / "nope")])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "source_not_found"


def test_invalid_plugin_name(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that a source path with an invalid name results in an error."""
    bad = tmp_path / "bad name"
    bad.mkdir()
    result = runner.invoke(cli_app, ["plugins", "install", str(bad)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "invalid_name"


def test_symlink_dir(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that a symlinked plugins directory results in an error."""
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    real = tmp_path / "real_plugins"
    real.mkdir()
    (tmp_path / "plugins").symlink_to(real)
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "symlink_dir"


def test_already_installed_without_force(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that installing an existing plugin fails without --force."""
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    (tmp_path / "plugins" / "plugin").mkdir(parents=True)
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "already_installed"


def test_remove_failed_on_force(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a failure during directory removal with --force is handled."""
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    (tmp_path / "plugins" / "plugin").mkdir(parents=True)
    monkeypatch.setattr(
        install_mod.shutil,  # type: ignore[attr-defined]
        "rmtree",
        lambda p: (_ for _ in ()).throw(RuntimeError("nope")),
    )
    result = runner.invoke(cli_app, ["plugins", "install", "--force", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "remove_failed"


def test_missing_plugin_py(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that a source directory missing plugin.py results in an error."""
    src = tmp_path / "plugin"
    src.mkdir()
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "plugin_py_missing"


def test_invalid_version_specifier(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that an invalid version specifier in plugin.py results in an error."""
    monkeypatch.setattr(install_mod, "parse_required_cli_version", lambda p: "no-spec")
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "invalid_specifier"


def test_disk_full(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that an ENOSPC OSError during copy is handled."""
    monkeypatch.setattr(
        install_mod.shutil,  # type: ignore[attr-defined]
        "copytree",
        lambda s, d, **kw: (_ for _ in ()).throw(OSError(errno.ENOSPC, "No space")),
    )
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "disk_full"


def test_permission_denied(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that an EACCES OSError during copy is handled."""
    monkeypatch.setattr(
        install_mod.shutil,  # type: ignore[attr-defined]
        "copytree",
        lambda s, d, **kw: (_ for _ in ()).throw(OSError(errno.EACCES, "Denied")),
    )
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "permission_denied"


def test_os_error_other(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a generic OSError during copy is handled."""
    monkeypatch.setattr(
        install_mod.shutil,  # type: ignore[attr-defined]
        "copytree",
        lambda s, d, **kw: (_ for _ in ()).throw(OSError(123, "oops")),
    )
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "os_error"


def test_plugin_py_missing_after_copy(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that an error is raised if plugin.py is missing after the copy."""

    def fake_copytree(src_arg: str, dst_arg: str, **kw: Any) -> None:
        Path(dst_arg).mkdir(parents=True)

    monkeypatch.setattr(
        install_mod.shutil,  # type: ignore[attr-defined]
        "copytree",
        fake_copytree,
    )
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "plugin_py_missing_after_copy"


def test_incompatible_cli_version(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that an incompatible CLI version specifier results in an error."""
    monkeypatch.setattr(install_mod, "parse_required_cli_version", lambda p: ">=2.0.0")

    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("#")
    result = runner.invoke(cli_app, ["plugins", "install", str(src)])

    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "invalid_specifier"


def test_dry_run(captured: dict[str, Any], runner: CliRunner, tmp_path: Path) -> None:
    """Test that --dry-run reports success without modifying the filesystem."""
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("# dry-run test")

    result = runner.invoke(cli_app, ["plugins", "install", "--dry-run", str(src)])
    assert result.exit_code == 0

    assert captured["payload"]["status"] == "dry-run"
    assert captured["payload"]["plugin"] == "plugin"
    assert not (tmp_path / "plugins" / "plugin").exists()


def test_successful_install_with_force(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that a successful installation with --force overwrites an existing plugin."""
    (tmp_path / "plugins" / "plugin").mkdir(parents=True)
    (tmp_path / "plugins" / "plugin" / "old.txt").write_text("old")

    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("# install-force test")

    result = runner.invoke(cli_app, ["plugins", "install", "--force", str(src)])
    assert result.exit_code == 0

    assert captured["payload"]["status"] == "installed"
    assert captured["payload"]["plugin"] == "plugin"
    assert (tmp_path / "plugins" / "plugin" / "plugin.py").is_file()


def test_src_resolve_fallback(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that the source path falls back to absolute() if resolve() fails."""
    monkeypatch.setattr(
        install_mod.Path,  # type: ignore[attr-defined]
        "resolve",
        lambda self: (_ for _ in ()).throw(OSError("fail resolve")),
    )

    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("# fallback test")

    rv = runner.invoke(cli_app, ["plugins", "install", "--dry-run", str(src)])
    assert rv.exit_code == 0

    assert captured["payload"]["source"] == str(src.absolute())


def test_force_removes_file_dest(
    captured: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that --force correctly removes a destination file before installing."""
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("# unlink test")

    plugins_dir = tmp_path / "plugins"
    plugins_dir.mkdir()
    dest_file = plugins_dir / "plugin"
    dest_file.write_text("stale")

    rv = runner.invoke(
        cli_app, ["plugins", "install", "--force", "--dry-run", str(src)]
    )
    assert rv.exit_code == 0

    assert not dest_file.exists()


def test_version_spec_success(
    captured: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a compatible version specifier allows installation."""
    monkeypatch.setattr(install_mod, "parse_required_cli_version", lambda p: ">=1.0.0")

    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("# version spec ok")

    rv = runner.invoke(cli_app, ["plugins", "install", "--dry-run", str(src)])
    assert rv.exit_code == 0

    assert captured["payload"]["status"] == "dry-run"


def test_plugins_dir_create_failure(
    runner: CliRunner,
    captured: dict[str, Any],
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a failure to create the plugins directory is handled."""
    src = tmp_path / "plugin"
    src.mkdir()
    (src / "plugin.py").write_text("# create-dir-fail test")

    orig_mkdir = install_mod.Path.mkdir  # type: ignore[attr-defined]

    def fail_plugins_mkdir(self: Path, *args: Any, **kwargs: Any) -> None:
        if self == tmp_path / "plugins":
            raise RuntimeError("mkdir boom")
        return orig_mkdir(self, *args, **kwargs)

    monkeypatch.setattr(install_mod.Path, "mkdir", fail_plugins_mkdir)  # type: ignore[attr-defined]

    result = runner.invoke(cli_app, ["plugins", "install", str(src)])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "create_dir_failed"
