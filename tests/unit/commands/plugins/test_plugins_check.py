# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins check module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from typing import Any
from unittest.mock import patch

import pytest

import bijux_cli.commands.plugins.check as plugin_check
from bijux_cli.commands.plugins.check import check_plugin


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

    monkeypatch.setattr(plugin_check, "emit_error_and_exit", fake_emit)


@pytest.mark.parametrize(
    ("ret", "expected"),
    [
        (True, "healthy"),
        (False, "unhealthy"),
        ({"status": "healthy"}, "healthy"),
    ],
)
def test_health_various_returns(
    tmp_path: Path, ret: bool | dict[str, str], expected: str
) -> None:
    """Test that various return types from a health hook are handled correctly."""
    code = f"def health(di): return {ret!r}\n"
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=True, py_code=code)
    with (
        patch("bijux_cli.commands.plugins.check.get_plugins_dir", lambda: root),
        patch("bijux_cli.commands.plugins.check.new_run_command") as mock_new_run,
    ):
        check_plugin(
            "foo", verbose=True, pretty=False, debug=True, fmt="json", quiet=False
        )
        payload = mock_new_run.call_args.kwargs["payload_builder"](True)
        assert payload["status"] == expected
        assert "python" in payload
        assert "platform" in payload
        exit_code = mock_new_run.call_args.kwargs["exit_code"]
        assert exit_code == (0 if expected == "healthy" else 1)


def test_missing_plugin_py(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a missing plugin.py file results in a 'not_found' error."""
    root = _make_dir(tmp_path, "foo", with_py=False, with_json=True)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)
    with pytest.raises(DummyExitError) as exc:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert exc.value.code == 1
    assert exc.value.payload["failure"] == "not_found"


def test_missing_metadata(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a missing plugin.json file results in a 'metadata_missing' error."""
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=False)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)
    with pytest.raises(DummyExitError) as exc:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert exc.value.code == 1
    assert exc.value.payload["failure"] == "metadata_missing"


def test_corrupt_metadata(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a corrupt plugin.json file results in a 'metadata_corrupt' error."""
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=False)
    (root / "foo" / "plugin.json").write_text(json.dumps(["oops"]))
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)
    with pytest.raises(DummyExitError) as exc:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert exc.value.payload["failure"] == "metadata_corrupt"
    assert (
        "Incomplete" in exc.value.payload["error"]
        or "corrupt" in exc.value.payload["error"]
    )


def test_import_spec_failure(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a failure to create an import spec results in an 'import_error'."""
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=True)

    def fake_spec(name: str, path: str) -> Any:
        return None

    monkeypatch.setattr(importlib.util, "spec_from_file_location", fake_spec)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)
    with pytest.raises(DummyExitError) as exc:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert exc.value.payload["failure"] == "import_error"
    assert "Cannot create import spec" in exc.value.payload["error"]


def test_import_exec_error_and_debug(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that an error during module execution is handled, with and without debug."""
    root = _make_dir(
        tmp_path, "foo", with_py=True, py_code="def oops(:\n", with_json=True
    )
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)

    with pytest.raises(DummyExitError) as exc1:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert exc1.value.payload["failure"] == "import_error"

    with pytest.raises(DummyExitError) as exc2:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=True
        )
    assert exc2.value.payload["error"].startswith("Import error")


def test_no_health_hook(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a missing health() hook in plugin.py results in a 'health_error'."""
    root = _make_dir(tmp_path, "foo", with_py=True, py_code="", with_json=True)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)
    with pytest.raises(DummyExitError) as exc:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert exc.value.payload["failure"] == "health_error"
    assert exc.value.payload["error"] == "No health() hook"


def test_bad_signature(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a health() hook with an incorrect signature results in an error."""
    code = "def health(a,b): return True\n"
    root = _make_dir(tmp_path, "foo", with_py=True, py_code=code, with_json=True)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)
    with pytest.raises(DummyExitError) as exc:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert exc.value.payload["failure"] == "health_error"
    assert "exactly one argument" in exc.value.payload["error"]


def test_health_raises(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an exception raised by a health() hook is handled correctly."""
    code = "def health(di): raise RuntimeError('boom')\n"
    root = _make_dir(tmp_path, "foo", with_py=True, py_code=code, with_json=True)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)
    with pytest.raises(DummyExitError) as exc:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )
    assert exc.value.payload["failure"] == "health_error"
    assert exc.value.payload["error"] == "boom"


def test_async_health_and_payload_builder(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test the successful execution of an asynchronous health() hook."""
    code = "async def health(di):\n    return {'status': 'healthy'}\n"
    root = _make_dir(tmp_path, "foo", with_py=True, py_code=code, with_json=True)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        plugin_check, "new_run_command", lambda **kw: captured.update(kw)
    )

    check_plugin(
        "foo", fmt="json", quiet=False, verbose=True, pretty=False, debug=False
    )

    assert captured["exit_code"] == 0
    builder = captured["payload_builder"]
    base = builder(False)
    assert base["plugin"] == "foo"
    assert base["status"] == "healthy"
    full = builder(True)
    assert "python" in full
    assert "platform" in full


def test_unexpected_health_return_marks_unhealthy(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that an unexpected return type from health() is marked as unhealthy."""
    code = "def health(di): return 123\n"
    root = _make_dir(tmp_path, "foo", with_py=True, py_code=code, with_json=True)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        plugin_check, "new_run_command", lambda **kw: captured.update(kw)
    )

    check_plugin("foo", fmt="json", quiet=False, verbose=True, pretty=True, debug=False)

    builder = captured["payload_builder"]
    payload = builder(False)
    assert payload["plugin"] == "foo"
    assert payload["status"] == "unhealthy"
    assert captured["exit_code"] == 1


def test_signature_introspection_error(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that an error during signature introspection of a health hook is handled."""
    code = "class Bad:\n    def __call__(self, di): return True\n    @property\n    def __signature__(self):\n        raise RuntimeError('sigfail')\nhealth = Bad()\n"
    root = _make_dir(tmp_path, "foo", with_py=True, py_code=code, with_json=True)
    monkeypatch.setattr(plugin_check, "get_plugins_dir", lambda: root)

    with pytest.raises(DummyExitError) as exc:
        check_plugin(
            "foo", fmt="json", quiet=False, verbose=False, pretty=False, debug=False
        )

    assert exc.value.payload["failure"] == "health_error"
    err = exc.value.payload["error"]
    assert err.startswith("health() signature error")
