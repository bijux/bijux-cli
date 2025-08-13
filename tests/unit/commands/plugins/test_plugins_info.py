# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins info module."""

# pyright: reportPrivateUsage=false

from __future__ import annotations

import json
from pathlib import Path
import platform
from typing import Any
from unittest.mock import patch

import pytest

import bijux_cli.commands.plugins.check as plugins_check
from bijux_cli.commands.plugins.check import check_plugin
import bijux_cli.commands.plugins.info as plugins_info
from bijux_cli.commands.plugins.info import _build_payload, info_plugin
from bijux_cli.commands.utilities import ascii_safe


class DummyExitError(Exception):
    """A custom exception to capture exit details in tests."""

    def __init__(self, code: int, payload: dict[str, Any]) -> None:
        """Initialize the DummyExit exception."""
        self.code = code
        self.payload = payload


def _make_dir(
    tmp_path: Path,
    name: str,
    *,
    with_py: bool = True,
    with_json: bool = True,
    py_code: str = "",
    meta: dict[str, Any] | None = None,
) -> Path:
    """Create a mock plugin directory structure."""
    root = tmp_path / "plugins"
    plugin = root / name
    plugin.mkdir(parents=True)
    if with_py:
        (plugin / "plugin.py").write_text(py_code)
    if with_json:
        data = meta if meta is not None else {"name": name, "desc": "d"}
        (plugin / "plugin.json").write_text(json.dumps(data))
    return root


def make_plugin_dir(
    tmp_path: Path,
    name: str,
    *,
    with_py: bool = True,
    with_json: bool = True,
    json_data: dict[str, Any] | None = None,
    py_code: str = "",
) -> Path:
    """Create a mock plugin directory."""
    plugin_dir = tmp_path / name
    plugin_dir.mkdir()
    if with_py:
        (plugin_dir / "plugin.py").write_text(py_code or "pass\n")
    if with_json:
        data = json_data if json_data is not None else {"name": name, "desc": "desc"}
        (plugin_dir / "plugin.json").write_text(json.dumps(data))
    return plugin_dir


def run_check(
    tmp_path: Path, name: str, fmt: str = "json", **opts: Any
) -> dict[str, Any]:
    """Run the check_plugin command with mocks and capture the result."""
    with patch("bijux_cli.commands.plugins.check.get_plugins_dir", lambda: tmp_path):
        captured: dict[str, Any] = {}
        with patch(
            "bijux_cli.commands.plugins.check.new_run_command",
            lambda **kw: captured.update(kw),
        ):
            check_plugin(
                name,
                quiet=opts.get("quiet", False),
                verbose=opts.get("verbose", False),
                fmt=fmt,
                pretty=opts.get("pretty", True),
                debug=opts.get("debug", False),
            )
        return captured


@pytest.fixture(autouse=True)
def _capture_emit(monkeypatch: pytest.MonkeyPatch) -> None:  # pyright: ignore[reportUnusedFunction]
    """Intercept error emissions and raise a custom exception."""

    def fake_emit(
        message: str,
        code: int,
        failure: str,
        command: str | None = None,
        fmt: str | None = None,
        quiet: bool = False,
        include_runtime: bool = False,
        debug: bool = False,
        extra: dict[str, Any] | None = None,
    ) -> None:
        payload = {"error": message, "failure": failure}
        if command:
            payload["command"] = command
        if fmt:
            payload["fmt"] = fmt
        if extra:
            payload.update(extra)
        raise DummyExitError(code, payload)

    monkeypatch.setattr(plugins_check, "emit_error_and_exit", fake_emit)


def test_info_plugin_not_found(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an error is raised when the plugin to be inspected is not found."""
    root = _make_dir(tmp_path, "foo", with_py=False, with_json=True)
    monkeypatch.setattr(plugins_info, "get_plugins_dir", lambda: root)

    def fake_emit(msg: str, code: int, failure: str, **kwargs: Any) -> None:
        raise DummyExitError(code, {"error": msg, "failure": failure})

    monkeypatch.setattr(plugins_info, "emit_error_and_exit", fake_emit)

    with pytest.raises(DummyExitError) as e:
        info_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert e.value.code == 1
    assert e.value.payload["failure"] == "not_found"
    assert "not found" in e.value.payload["error"].lower()


def test_info_metadata_corrupt_json(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that an error is raised for a plugin with corrupt metadata JSON."""
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=False)
    plugin_dir = root / "foo"
    (plugin_dir / "plugin.json").write_text('{"name": "foo", "desc": }')
    monkeypatch.setattr(plugins_info, "get_plugins_dir", lambda: root)

    def fake_emit(msg: str, code: int, failure: str, **kwargs: Any) -> None:
        raise DummyExitError(code, {"error": msg, "failure": failure})

    monkeypatch.setattr(plugins_info, "emit_error_and_exit", fake_emit)

    with pytest.raises(DummyExitError) as e:
        info_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )

    assert e.value.payload["failure"] == "metadata_corrupt"
    assert "corrupt" in e.value.payload["error"].lower()


def test_info_metadata_missing_field(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that an error is raised for a plugin with missing required metadata fields."""
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=True, meta={"desc": "d"})
    monkeypatch.setattr(plugins_info, "get_plugins_dir", lambda: root)

    def fake_emit(msg: str, code: int, failure: str, **kwargs: Any) -> None:
        raise DummyExitError(code, {"error": msg, "failure": failure})

    monkeypatch.setattr(plugins_info, "emit_error_and_exit", fake_emit)

    with pytest.raises(DummyExitError) as e:
        info_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )

    assert e.value.payload["failure"] == "metadata_corrupt"
    assert "missing required" in e.value.payload["error"].lower()


def test_info_success_without_runtime(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test the successful retrieval of plugin info without runtime data."""
    meta = {"name": "foo", "desc": "desc", "version": "1.0"}
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=True, meta=meta)
    monkeypatch.setattr(plugins_info, "get_plugins_dir", lambda: root)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        plugins_info, "new_run_command", lambda **kw: captured.update(kw)
    )

    info_plugin("foo", fmt="yaml", quiet=True, verbose=False, pretty=True, debug=False)

    assert captured["command_name"] == "plugins info"
    assert captured["fmt"] == "yaml"
    builder = captured["payload_builder"]
    base = builder(False)
    assert base["name"] == "foo"
    assert base["desc"] == "desc"
    assert base["version"] == "1.0"
    assert "python" not in base
    assert "platform" not in base


def test_info_success_with_runtime(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test the successful retrieval of plugin info with runtime data."""
    meta = {"name": "foo", "desc": "d"}
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=True, meta=meta)
    monkeypatch.setattr(plugins_info, "get_plugins_dir", lambda: root)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        plugins_info, "new_run_command", lambda **kw: captured.update(kw)
    )

    info_plugin("foo", fmt="JSON", quiet=False, verbose=True, pretty=False, debug=False)

    builder = captured["payload_builder"]
    full = builder(True)
    assert full["python"] == ascii_safe(platform.python_version(), "python_version")
    assert full["platform"] == ascii_safe(platform.platform(), "platform")


def test_build_payload_directly() -> None:
    """Test the _build_payload utility for info commands directly."""
    payload = {"foo": "bar"}
    no_rt = _build_payload(False, payload.copy())
    assert no_rt == {"foo": "bar"}
    with_rt = _build_payload(True, payload.copy())
    assert "python" in with_rt
    assert "platform" in with_rt


def test_info_no_metadata_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test retrieving info for a plugin that is missing its metadata file."""
    root = tmp_path / "plugins"
    plugin = root / "foo"
    plugin.mkdir(parents=True)
    (plugin / "plugin.py").write_text("pass\n")
    monkeypatch.setattr(plugins_info, "get_plugins_dir", lambda: root)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        plugins_info, "new_run_command", lambda **kw: captured.update(kw)
    )

    info_plugin("foo", fmt="json", quiet=False, verbose=False, pretty=True, debug=False)

    builder = captured["payload_builder"]
    payload = builder(False)
    assert payload == {"name": "foo", "path": str(plugin)}
