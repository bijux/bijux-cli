# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins check module."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from typing import Any
from unittest.mock import patch

import pytest

import bijux_cli.cli.plugins.commands.check as plugin_check
from bijux_cli.cli.plugins.commands.check import check_plugin
from bijux_cli.core.enums import LogLevel
from bijux_cli.core.precedence import default_execution_policy
from bijux_cli.core.runtime import run_command
from bijux_cli.plugins.metadata import PluginMetadata, PluginMetadataError


class DummyExitError(Exception):
    """A custom exception to capture exit details in tests."""

    def __init__(self, code: int, payload: dict[str, Any]) -> None:
        """Initialize the DummyExit exception."""
        self.code = code
        self.payload = payload


@pytest.fixture(autouse=True)
def _default_policy(monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensure a default execution policy for CLI output helpers."""
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: default_execution_policy(),
    )


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
        data = (
            meta
            if meta is not None
            else {
                "name": name,
                "desc": "d",
                "bijux_cli_version": ">=0.1.0",
            }
        )
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
        data = (
            json_data
            if json_data is not None
            else {
                "name": name,
                "desc": "desc",
                "bijux_cli_version": ">=0.1.0",
            }
        )
        (plugin_dir / "plugin.json").write_text(json.dumps(data))
    return plugin_dir


def run_check(
    tmp_path: Path, name: str, fmt: str = "json", **opts: Any
) -> dict[str, Any]:
    """Run the check_plugin command with mocks and capture the result."""
    meta = PluginMetadata(
        name=name,
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=tmp_path / name,
    )
    with patch(
        "bijux_cli.cli.plugins.commands.check.get_plugin_metadata", lambda _: meta
    ):
        captured: dict[str, Any] = {}
        with (
            patch(
                "bijux_cli.cli.plugins.commands.check.new_run_command",
                lambda **kw: captured.update(kw),
            ),
            patch(
                "bijux_cli.cli.core.command.current_execution_policy",
                lambda: default_execution_policy(),
            ),
        ):
            run_command(
                check_plugin,
                name,
                quiet=opts.get("quiet", False),
                fmt=fmt,
                pretty=opts.get("pretty", True),
                log_level=opts.get("log_level", "info"),
            )
        return captured


@pytest.fixture(autouse=True)
def _capture_emit(monkeypatch: pytest.MonkeyPatch) -> None:
    """Intercept error emissions and raise a custom exception."""

    def fake_emit(
        message: str,
        code: int,
        failure: str,
        command: str | None = None,
        fmt: str | None = None,
        quiet: bool = False,
        include_runtime: bool = False,
        log_level: Any | None = None,
        extra: dict[str, Any] | None = None,
        **_kwargs: Any,
    ) -> None:
        payload = {"error": message, "failure": failure}
        if command:
            payload["command"] = command
        if fmt:
            payload["fmt"] = fmt
        if extra:
            payload.update(extra)
        raise DummyExitError(code, payload)

    monkeypatch.setattr(plugin_check, "raise_exit_intent", fake_emit)


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
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    with (
        patch(
            "bijux_cli.cli.plugins.commands.check.get_plugin_metadata", lambda _: meta
        ),
        patch("bijux_cli.cli.plugins.commands.check.new_run_command") as mock_new_run,
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            lambda: default_execution_policy(),
        ),
    ):
        run_command(
            check_plugin,
            "foo",
            pretty=False,
            log_level=LogLevel.DEBUG,
            fmt="json",
            quiet=False,
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
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)
    with pytest.raises(DummyExitError) as exc:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )
    assert exc.value.code == 1
    assert exc.value.payload["failure"] == "not_found"


def test_missing_metadata(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that missing metadata results in a metadata error."""
    monkeypatch.setattr(
        plugin_check,
        "get_plugin_metadata",
        lambda _: (_ for _ in ()).throw(PluginMetadataError("missing")),
    )
    with pytest.raises(DummyExitError) as exc:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )
    assert exc.value.code == 1
    assert exc.value.payload["failure"] == "metadata_error"


def test_corrupt_metadata(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that corrupt metadata results in a metadata error."""
    monkeypatch.setattr(
        plugin_check,
        "get_plugin_metadata",
        lambda _: (_ for _ in ()).throw(PluginMetadataError("corrupt")),
    )
    with pytest.raises(DummyExitError) as exc:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )
    assert exc.value.payload["failure"] == "metadata_error"


def test_import_spec_failure(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a failure to create an import spec results in an 'import_error'."""
    root = _make_dir(tmp_path, "foo", with_py=True, with_json=True)

    def fake_spec(name: str, path: str) -> Any:
        return None

    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(importlib.util, "spec_from_file_location", fake_spec)
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)
    with pytest.raises(DummyExitError) as exc:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
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
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)

    with pytest.raises(DummyExitError) as exc1:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )
    assert exc1.value.payload["failure"] == "import_error"

    with pytest.raises(DummyExitError) as exc2:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.DEBUG,
        )
    assert exc2.value.payload["error"].startswith("Import error")


def test_no_health_hook(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a missing health() hook in plugin.py results in a 'health_error'."""
    root = _make_dir(tmp_path, "foo", with_py=True, py_code="", with_json=True)
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)
    with pytest.raises(DummyExitError) as exc:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )
    assert exc.value.payload["failure"] == "health_error"
    assert exc.value.payload["error"] == "No health() hook"


def test_bad_signature(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a health() hook with an incorrect signature results in an error."""
    code = "def health(a,b): return True\n"
    root = _make_dir(tmp_path, "foo", with_py=True, py_code=code, with_json=True)
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)
    with pytest.raises(DummyExitError) as exc:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )
    assert exc.value.payload["failure"] == "health_error"
    assert "exactly one argument" in exc.value.payload["error"]


def test_health_raises(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an exception raised by a health() hook is handled correctly."""
    code = "def health(di): raise RuntimeError('boom')\n"
    root = _make_dir(tmp_path, "foo", with_py=True, py_code=code, with_json=True)
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)
    with pytest.raises(DummyExitError) as exc:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )
    assert exc.value.payload["failure"] == "health_error"
    assert exc.value.payload["error"] == "boom"


def test_async_health_and_payload_builder(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test the successful execution of an asynchronous health() hook."""
    code = "async def health(di):\n    return {'status': 'healthy'}\n"
    root = _make_dir(tmp_path, "foo", with_py=True, py_code=code, with_json=True)
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        plugin_check, "new_run_command", lambda **kw: captured.update(kw)
    )

    run_command(
        check_plugin,
        "foo",
        fmt="json",
        quiet=False,
        pretty=False,
        log_level=LogLevel.INFO,
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
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        plugin_check, "new_run_command", lambda **kw: captured.update(kw)
    )

    run_command(
        check_plugin,
        "foo",
        fmt="json",
        quiet=False,
        pretty=True,
        log_level=LogLevel.INFO,
    )

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
    meta = PluginMetadata(
        name="foo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0.1.0",
        path=root / "foo",
    )
    monkeypatch.setattr(plugin_check, "get_plugin_metadata", lambda _: meta)

    with pytest.raises(DummyExitError) as exc:
        run_command(
            check_plugin,
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )

    assert exc.value.payload["failure"] == "health_error"
    err = exc.value.payload["error"]
    assert err.startswith("health() signature error")
