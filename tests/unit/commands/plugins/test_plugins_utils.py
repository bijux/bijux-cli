# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins utils module."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import pytest

from bijux_cli.commands.plugins.utils import (
    ignore_hidden_and_broken_symlinks,
    parse_required_cli_version,
    refuse_on_symlink,
)
import bijux_cli.commands.utilities as utilities_mod


def test_ignore_hidden_and_broken_symlinks(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test the filtering of hidden files and broken symlinks."""
    d = tmp_path / "d"
    d.mkdir()
    (d / "a.txt").write_text("ok")
    (d / ".secret").write_text("hidden")
    (d / "plugin.py").write_text("# plugin")
    target = d / "a.txt"
    (d / "good_link").symlink_to(target)
    (d / "broken_link").symlink_to(d / "nonexistent")

    names = sorted(name for name in os.listdir(d))
    skipped = ignore_hidden_and_broken_symlinks(str(d), names)

    assert "a.txt" not in skipped
    assert "plugin.py" not in skipped
    assert "good_link" not in skipped
    assert ".secret" in skipped
    assert "broken_link" in skipped


def test_parse_required_cli_version_absent(tmp_path: Path) -> None:
    """Test that parsing returns None when no version specifier is present."""
    p = tmp_path / "plugin.py"
    p.write_text("# no requirements here\nfoo = 1\n")
    assert parse_required_cli_version(p) is None


def test_parse_required_cli_version_on_errors(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that parsing handles various errors gracefully by returning None."""
    p = tmp_path / "bad.py"
    p.write_text("def f(:\n")
    assert parse_required_cli_version(p) is None

    assert parse_required_cli_version(tmp_path / "nope.py") is None

    p2 = tmp_path / "unreadable.py"
    p2.write_text('requires_cli_version = "x"')
    monkeypatch.setattr(
        Path, "open", lambda *args, **kwargs: (_ for _ in ()).throw(PermissionError())
    )
    assert parse_required_cli_version(p2) is None


def test_refuse_on_symlink_noop(tmp_path: Path) -> None:
    """Test that the symlink check does nothing for a regular directory."""
    d = tmp_path / "normal"
    d.mkdir()
    refuse_on_symlink(d, "plugins install", "json", False, False, False)


def test_refuse_on_symlink_calls_emit(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that a symlinked directory triggers a structured error."""
    real = tmp_path / "real"
    real.mkdir()
    link = tmp_path / "link"
    link.symlink_to(real, target_is_directory=True)

    called: dict[str, Any] = {}

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
        called.update(
            message=msg,
            code=code,
            failure=failure,
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )
        raise SystemExit(code)

    monkeypatch.setattr(utilities_mod, "emit_error_and_exit", fake_emit)

    with pytest.raises(SystemExit) as exc:
        refuse_on_symlink(link, "plugins uninstall", "yaml", True, True, True)

    assert exc.value.code == 1
    assert called["failure"] == "symlink_dir"
    assert "Refusing to uninstall" in called["message"]
    assert "'link'" in called["message"]


def test_parse_required_cli_version_class_attr(tmp_path: Path) -> None:
    """Test parsing a version specifier defined as a class attribute."""
    plugin_py = tmp_path / "plugin.py"
    plugin_py.write_text('class Plugin:\n    requires_cli_version = ">=2.5.0"\n')
    spec = parse_required_cli_version(plugin_py)
    assert spec == ">=2.5.0"


def test_parse_required_cli_version_class_attr_field_first(tmp_path: Path) -> None:
    """Test that the correct class attribute is parsed among others."""
    plugin_py = tmp_path / "plugin.py"
    plugin_py.write_text(
        'class Plugin:\n    something_else = 123\n    requires_cli_version = "<3.0.0"\n'
    )
    spec = parse_required_cli_version(plugin_py)
    assert spec == "<3.0.0"


def test_parse_required_cli_version_no_specifier(tmp_path: Path) -> None:
    """Test that parsing a file with no specifier returns None."""
    plugin_py = tmp_path / "plugin.py"
    plugin_py.write_text("def foo(): pass\n")
    assert parse_required_cli_version(plugin_py) is None


def test_parse_required_cli_version_syntax_error(tmp_path: Path) -> None:
    """Test that a syntax error in the parsed file is handled gracefully."""
    plugin_py = tmp_path / "plugin.py"
    plugin_py.write_text("this is not valid python!!!")
    assert parse_required_cli_version(plugin_py) is None


def test_parse_required_cli_version_top_level(tmp_path: Path) -> None:
    """Test parsing a version specifier defined at the module level."""
    plugin_py = tmp_path / "plugin.py"
    plugin_py.write_text('requires_cli_version = "~=1.2.3"\n')
    spec = parse_required_cli_version(plugin_py)
    assert spec == "~=1.2.3"


def test_parse_required_cli_version_class_level_only(tmp_path: Path) -> None:
    """Test parsing a file that only contains a class-level specifier."""
    plugin_py = tmp_path / "plugin.py"
    plugin_py.write_text('class Plugin:\n    requires_cli_version = ">=2.5.0"\n')
    spec = parse_required_cli_version(plugin_py)
    assert spec == ">=2.5.0"


def test_parse_required_cli_version_prefers_top_level(tmp_path: Path) -> None:
    """Test that a top-level specifier is preferred over a class-level one."""
    plugin_py = tmp_path / "plugin.py"
    plugin_py.write_text(
        'requires_cli_version = "1.0.0"\n\nclass Plugin:\n    requires_cli_version = ">=2.5.0"\n'
    )
    spec = parse_required_cli_version(plugin_py)
    assert spec == "1.0.0"


def test_parse_required_cli_version_plugin_class_exists_without_specifier(
    tmp_path: Path,
) -> None:
    """Test parsing a file with a Plugin class that lacks a version specifier."""
    plugin_py = tmp_path / "plugin.py"
    plugin_py.write_text(
        'class Plugin:\n    description = "A test plugin."\n\n    def execute(self):\n        return "done"\n'
    )
    spec = parse_required_cli_version(plugin_py)
    assert spec is None
