# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI version & plugin surface."""

from __future__ import annotations

from collections.abc import Callable
import importlib
import io
from pathlib import Path
from typing import IO, Any

from packaging.version import Version
import pytest

import bijux_cli.plugins as plugins

version_mod = importlib.import_module("bijux_cli.core.version")


def _reload_with(
    monkeypatch: pytest.MonkeyPatch,
    *,
    metadata_version: Callable[[str], str] | None = None,
    __file__: str | None = None,
    pyproject_bytes: bytes | None = None,
) -> None:
    """Reload ``bijux_cli.core.version`` under controlled conditions.

    Args:
        monkeypatch: Pytest monkeypatch fixture.
        metadata_version: Replacement for ``importlib.metadata.version``.
        __file__: Fake module file path to influence where ``pyproject.toml`` is read.
        pyproject_bytes: Bytes to return when opening any ``pyproject.toml`` path.
    """
    if metadata_version is not None:
        monkeypatch.setattr(
            "importlib.metadata.version", metadata_version, raising=False
        )

    if __file__ is not None:
        monkeypatch.setattr(version_mod, "__file__", __file__)

    if pyproject_bytes is not None:
        real_open = Path.open

        def _fake_open(
            self: Path, mode: str = "r", *args: Any, **kwargs: Any
        ) -> IO[Any]:
            if self.name == "pyproject.toml" and "b" in mode:
                return io.BytesIO(pyproject_bytes)
            return real_open(self, mode, *args, **kwargs)

        monkeypatch.setattr(Path, "open", _fake_open, raising=True)

    importlib.reload(version_mod)


def test_version_from_package_metadata(monkeypatch: pytest.MonkeyPatch) -> None:
    """Version is loaded from package metadata when available."""
    _reload_with(monkeypatch, metadata_version=lambda name: "1.2.3")
    assert version_mod.__version__ == "1.2.3"
    assert isinstance(version_mod.version, Version)
    assert str(version_mod.version) == "1.2.3"


def test_version_fallback_when_package_not_found(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Version falls back to default when package metadata is missing."""
    from importlib.metadata import PackageNotFoundError

    def _raise(_: str) -> str:
        raise PackageNotFoundError

    _reload_with(monkeypatch, metadata_version=_raise)
    assert version_mod.__version__ == "0.1.0"
    assert isinstance(version_mod.version, Version)
    assert str(version_mod.version) == "0.1.0"


def test_api_version_from_pyproject(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """API version is read from ``[tool.bijux]`` section in pyproject.toml."""
    fake_file = str(tmp_path / "src" / "bijux_cli" / "core" / "version.py")
    pyproject = b"""
[tool.bijux]
api_version = "2.5.0"
"""
    _reload_with(
        monkeypatch,
        __file__=fake_file,
        pyproject_bytes=pyproject,
        metadata_version=lambda _: "9.9.9",
    )

    assert version_mod.__api_version__ == "2.5.0"
    assert isinstance(version_mod.api_version, Version)
    assert str(version_mod.api_version) == "2.5.0"


def test_api_version_fallback_when_file_not_found(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """API version falls back when ``pyproject.toml`` is not found."""
    _reload_with(
        monkeypatch,
        __file__="/non/existent/path/__version__.py",
        metadata_version=lambda _: "9.9.9",
    )
    assert version_mod.__api_version__ == "0.1.0"
    assert isinstance(version_mod.api_version, Version)
    assert str(version_mod.api_version) == "0.1.0"


def test_api_version_fallback_when_key_missing(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """API version falls back when key is missing from pyproject.toml."""
    fake_file = str(tmp_path / "src" / "bijux_cli" / "__version__.py")
    pyproject = b"""
[tool.another_tool]
api_version = "3.0.0"
"""
    _reload_with(
        monkeypatch,
        __file__=fake_file,
        pyproject_bytes=pyproject,
        metadata_version=lambda _: "9.9.9",
    )
    assert version_mod.__api_version__ == "0.1.0"
    assert isinstance(version_mod.api_version, Version)
    assert str(version_mod.api_version) == "0.1.0"


def test_all_exports_are_present() -> None:
    """``__all__`` contains the expected public symbols."""
    expected = {"version", "api_version", "__version__", "__api_version__"}
    assert set(version_mod.__all__) == expected


def test_getattr_raises_attribute_error() -> None:
    """Missing plugin attribute raises a clear AttributeError."""
    attr = "non_existent_attribute"
    with pytest.raises(AttributeError) as exc:
        getattr(plugins, attr)

    msg = str(exc.value)
    assert "bijux_cli.plugins" in msg
    assert "has no attribute" in msg
    assert attr in msg
