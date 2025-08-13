# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins scaffold module."""

# mypy: disable-error-code="union-attr"
# pyright: reportOptionalMemberAccess=false
# pyright: reportAttributeAccessIssue=false

from __future__ import annotations

import builtins
import json
from pathlib import Path
import sys
from types import ModuleType
from typing import Any

import pytest
from typer.testing import CliRunner

from bijux_cli.cli import app as cli_app
import bijux_cli.commands.plugins.scaffold as scaffold_mod


@pytest.fixture
def cap(monkeypatch: pytest.MonkeyPatch) -> dict[str, Any]:
    """Stub out I/O and capture final payloads and errors."""
    data: dict[str, Any] = {}

    monkeypatch.setattr(
        scaffold_mod,
        "validate_common_flags",
        lambda fmt, cmd, quiet: fmt.lower(),
    )

    def fake_new_run(**kwargs: Any) -> None:
        data["command"] = kwargs["command_name"]
        data["payload"] = kwargs["payload_builder"](include=True)
        for flag in ("quiet", "verbose", "fmt", "pretty", "debug"):
            data[flag] = kwargs.get(flag)

    monkeypatch.setattr(scaffold_mod, "new_run_command", fake_new_run)

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
        raise RuntimeError({"failure": failure, "message": msg})

    monkeypatch.setattr(scaffold_mod, "emit_error_and_exit", fake_emit)
    return data


@pytest.fixture
def runner() -> CliRunner:
    """Provide a CliRunner instance."""
    return CliRunner()


def test_reserved_keyword(cap: dict[str, Any], runner: CliRunner) -> None:
    """Test that using a Python reserved keyword as a plugin name fails."""
    result = runner.invoke(cli_app, ["plugins", "scaffold", "for", "-t", "tpl"])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "reserved_keyword"


def test_invalid_name(cap: dict[str, Any], runner: CliRunner) -> None:
    """Test that using an invalid Python identifier as a plugin name fails."""
    result = runner.invoke(cli_app, ["plugins", "scaffold", "bad name", "-t", "tpl"])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "invalid_name"


def test_no_template(cap: dict[str, Any], runner: CliRunner) -> None:
    """Test that not providing a template fails."""
    result = runner.invoke(cli_app, ["plugins", "scaffold", "plugin"])
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "no_template"


def test_parent_mkdir_failed(
    cap: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a failure to create the parent output directory is handled."""
    parent = tmp_path / "newparent"
    src_name = "plugin"
    orig = Path.mkdir

    def fail_once(self: Path, *args: Any, **kwargs: Any) -> None:
        if self == parent:
            raise RuntimeError("boom")
        return orig(self, *args, **kwargs)

    monkeypatch.setattr(Path, "mkdir", fail_once)

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", src_name, "-o", str(parent), "-t", "tpl"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "create_dir_failed"


def test_parent_not_dir(cap: dict[str, Any], runner: CliRunner, tmp_path: Path) -> None:
    """Test that specifying a file as the parent output directory fails."""
    parent = tmp_path / "afile"
    parent.write_text("x")
    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "not_dir"


def test_name_conflict(cap: dict[str, Any], runner: CliRunner, tmp_path: Path) -> None:
    """Test that a case-insensitive name conflict in the output directory fails."""
    parent = tmp_path / "out"
    parent.mkdir()
    (parent / "Plugin").mkdir()
    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "name_conflict"


def test_dir_not_empty(cap: dict[str, Any], runner: CliRunner, tmp_path: Path) -> None:
    """Test that scaffolding into a non-empty directory fails without --force."""
    parent = tmp_path / "out"
    parent.mkdir()
    (parent / "plugin").mkdir()
    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "dir_not_empty"


def test_remove_failed(
    cap: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a failure during the removal of an existing directory is handled."""
    parent = tmp_path / "out"
    parent.mkdir()
    target = parent / "plugin"
    target.mkdir()
    monkeypatch.setattr(
        scaffold_mod.shutil,  # type: ignore[attr-defined]
        "rmtree",
        lambda p: (_ for _ in ()).throw(RuntimeError("rm fail")),
    )
    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "remove_failed"


def test_cookiecutter_missing(
    cap: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a missing cookiecutter dependency is handled."""
    parent = tmp_path / "out"
    parent.mkdir()
    orig_import = builtins.__import__

    def fake_import(
        name: str,
        globals_: dict[str, Any] | None = None,
        locals_: dict[str, Any] | None = None,
        fromlist: tuple[str, ...] = (),
        level: int = 0,
    ) -> Any:
        if name.startswith("cookiecutter"):
            raise ModuleNotFoundError
        return orig_import(name, globals_, locals_, fromlist, level)

    monkeypatch.setattr(builtins, "__import__", fake_import)

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "cookiecutter_missing"


def test_scaffold_failed(
    cap: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a failure during the cookiecutter scaffolding process is handled."""
    parent = tmp_path / "out"
    parent.mkdir()

    m = ModuleType("cookiecutter.main")

    def bad_cc(
        template: str, no_input: bool, output_dir: str, extra_context: dict[str, Any]
    ) -> None:
        raise RuntimeError("bad template")

    m.cookiecutter = bad_cc  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "cookiecutter.main", m)
    monkeypatch.setitem(sys.modules, "cookiecutter", ModuleType("cookiecutter"))

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "scaffold_failed"


def test_plugin_json_missing(
    cap: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a missing plugin.json after scaffolding results in an error."""
    parent = tmp_path / "out"
    parent.mkdir()

    m = ModuleType("cookiecutter.main")

    def make_dir(
        template: str, no_input: bool, output_dir: str, extra_context: dict[str, Any]
    ) -> None:
        tgt = Path(output_dir) / extra_context["project_name"]
        tgt.mkdir()

    m.cookiecutter = make_dir  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "cookiecutter.main", m)
    monkeypatch.setitem(sys.modules, "cookiecutter", ModuleType("cookiecutter"))

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "plugin_json_missing"


def test_plugin_json_invalid(
    cap: dict[str, Any],
    runner: CliRunner,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that an invalid plugin.json after scaffolding results in an error."""
    parent = tmp_path / "out"
    parent.mkdir()

    m = ModuleType("cookiecutter.main")

    def make_bad_json(
        template: str, no_input: bool, output_dir: str, extra_context: dict[str, Any]
    ) -> None:
        tgt = Path(output_dir) / extra_context["project_name"]
        tgt.mkdir()
        (tgt / "plugin.json").write_text(
            json.dumps({"name": extra_context["project_name"]})
        )

    m.cookiecutter = make_bad_json  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "cookiecutter.main", m)
    monkeypatch.setitem(sys.modules, "cookiecutter", ModuleType("cookiecutter"))

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 1
    assert result.exception.args[0]["failure"] == "plugin_json_invalid"


def test_success(cap: dict[str, Any], runner: CliRunner, tmp_path: Path) -> None:
    """Test the successful scaffolding of a new plugin."""
    parent = tmp_path / "out"
    parent.mkdir()

    m = ModuleType("cookiecutter.main")

    def make_good(
        template: str, no_input: bool, output_dir: str, extra_context: dict[str, Any]
    ) -> None:
        tgt = Path(output_dir) / extra_context["project_name"]
        tgt.mkdir()
        (tgt / "plugin.json").write_text(
            json.dumps(
                {"name": extra_context["project_name"], "desc": "some description"}
            )
        )

    m.cookiecutter = make_good  # type: ignore[attr-defined]
    sys.modules["cookiecutter.main"] = m
    sys.modules["cookiecutter"] = ModuleType("cookiecutter")

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 0

    assert cap["payload"] == {
        "status": "created",
        "plugin": "plugin",
        "dir": str(parent / "plugin"),
    }


def _stub_cookiecutter_creates_good_dir() -> None:
    """Inject a fake cookiecutter that creates a valid plugin directory."""
    m = ModuleType("cookiecutter.main")

    def cc(
        template: str, no_input: bool, output_dir: str, extra_context: dict[str, Any]
    ) -> None:
        tgt = Path(output_dir) / extra_context["project_name"]
        tgt.mkdir()
        (tgt / "plugin.json").write_text(
            json.dumps(
                {"name": extra_context["project_name"], "desc": "valid description"}
            )
        )

    m.cookiecutter = cc  # type: ignore[attr-defined]
    sys.modules["cookiecutter.main"] = m
    sys.modules["cookiecutter"] = ModuleType("cookiecutter")


def test_force_removes_existing_symlink(
    cap: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that --force correctly removes an existing symlink."""
    parent = tmp_path / "out"
    parent.mkdir()
    target = parent / "plugin"
    real = tmp_path / "real"
    real.mkdir()
    target.symlink_to(real)

    _stub_cookiecutter_creates_good_dir()

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 0
    assert (parent / "plugin").is_dir()
    assert cap["payload"]["status"] == "created"
    assert cap["payload"]["plugin"] == "plugin"


def test_force_removes_existing_directory(
    cap: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that --force correctly removes an existing directory."""
    parent = tmp_path / "out"
    parent.mkdir()
    target = parent / "plugin"
    target.mkdir()
    (target / "old.txt").write_text("old")

    _stub_cookiecutter_creates_good_dir()

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 0
    assert (parent / "plugin").is_dir()
    assert cap["payload"]["status"] == "created"


def test_scaffold_fails_if_template_does_not_create_dir(
    cap: dict[str, Any], runner: CliRunner, tmp_path: Path
) -> None:
    """Test that a failure is reported if cookiecutter does not create the target directory."""
    parent = tmp_path / "out"
    parent.mkdir()
    m = ModuleType("cookiecutter.main")
    m.cookiecutter = lambda *args, **kwargs: None  # type: ignore[attr-defined]
    sys.modules["cookiecutter.main"] = m
    sys.modules["cookiecutter"] = ModuleType("cookiecutter")

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl"],
    )
    assert result.exit_code == 1
    assert isinstance(result.exception, RuntimeError)
    assert result.exception.args[0]["failure"] == "scaffold_failed"


@pytest.fixture
def captured() -> dict[str, Any]:
    """Provide a dictionary to capture test outputs."""
    return {}


@pytest.fixture(autouse=True)
def _stub_out(  # pyright: ignore[reportUnusedFunction]
    monkeypatch: pytest.MonkeyPatch, captured: dict[str, Any]
) -> None:
    """Stub out command runners and error emitters for tests."""

    def fake_new_run_command(**kwargs: Any) -> None:
        captured["command_name"] = kwargs["command_name"]
        captured["payload"] = kwargs["payload_builder"](True)
        captured["opts"] = {
            "quiet": kwargs["quiet"],
            "verbose": kwargs["verbose"],
            "fmt": kwargs["fmt"],
            "pretty": kwargs["pretty"],
            "debug": kwargs["debug"],
        }

    monkeypatch.setattr(scaffold_mod, "new_run_command", fake_new_run_command)

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

    monkeypatch.setattr(scaffold_mod, "emit_error_and_exit", fake_emit_error_and_exit)


def _inject_cookiecutter(fn: Any) -> None:
    """Inject a fake cookiecutter function into sys.modules."""
    m = ModuleType("cookiecutter.main")
    m.cookiecutter = fn  # type: ignore[attr-defined]
    sys.modules["cookiecutter.main"] = m
    sys.modules["cookiecutter"] = ModuleType("cookiecutter")


def test_force_removes_existing_file(
    tmp_path: Path, runner: CliRunner, captured: dict[str, Any]
) -> None:
    """Test that --force correctly removes an existing file."""
    parent = tmp_path / "out"
    parent.mkdir()
    stale_file = parent / "plugin"
    stale_file.write_text("stale")

    def make_good(
        template: str, no_input: bool, output_dir: str, extra_context: dict[str, Any]
    ) -> None:
        new_dir = Path(output_dir) / extra_context["project_name"]
        new_dir.mkdir()
        (new_dir / "plugin.json").write_text(
            json.dumps({"name": extra_context["project_name"], "desc": "description"})
        )

    _inject_cookiecutter(make_good)

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "plugin", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 0

    final_dir = parent / "plugin"
    assert final_dir.is_dir()
    assert (final_dir / "plugin.json").is_file()

    assert captured["payload"]["status"] == "created"
    assert captured["payload"]["plugin"] == "plugin"


def test_template_copy_failed(
    tmp_path: Path, runner: CliRunner, captured: dict[str, Any]
) -> None:
    """Test that a failure during template copying is handled."""
    parent = tmp_path / "out"
    parent.mkdir()

    def do_nothing(
        template: str, no_input: bool, output_dir: str, extra_context: dict[str, Any]
    ) -> None:
        return

    _inject_cookiecutter(do_nothing)

    result = runner.invoke(
        cli_app,
        ["plugins", "scaffold", "myplug", "-o", str(parent), "-t", "tpl", "--force"],
    )
    assert result.exit_code == 1
    err = result.exception
    assert isinstance(err, RuntimeError)
    info = err.args[0]
    assert info["failure"] == "scaffold_failed"
    assert "Template copy failed" in info["message"]
