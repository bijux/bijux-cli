# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI root init module."""

from __future__ import annotations

import importlib

from packaging.version import Version
import pytest

import bijux_cli


def test___all___exports() -> None:
    """Test that the package's __all__ attribute exports the correct names."""
    assert set(bijux_cli.__all__) == {
        "version",
        "api_version",
        "BijuxAPI",
        "entry_point",
    }


def test_version_and_api_version_are_strings_or_versions() -> None:
    """Test that version and api_version are valid string or Version objects."""
    for attr in ("version", "api_version"):
        val = getattr(bijux_cli, attr)
        assert isinstance(val, str | Version)
        s = str(val)
        assert "." in s
        major = s.split(".", 1)[0]
        assert major.isdigit()


def test_bijux_api_class_exported() -> None:
    """Test that BijuxAPI is correctly exported from the top-level package."""
    from bijux_cli.api import BijuxAPI as ImplBijuxAPI

    assert bijux_cli.BijuxAPI is ImplBijuxAPI


@pytest.mark.parametrize(
    ("ret", "expected"),
    [
        (0, 0),
        (42, 42),
        (None, None),
    ],
)
def test_entry_point_returns_main_return(
    monkeypatch: pytest.MonkeyPatch, ret: int | None, expected: int | None
) -> None:
    """Test that the entry_point function returns the result of main()."""
    monkeypatch.setattr(bijux_cli, "main", lambda: ret)
    assert bijux_cli.entry_point() == expected


@pytest.mark.parametrize("code", [0, 1, 5, None])
def test_entry_point_catches_system_exit(
    monkeypatch: pytest.MonkeyPatch, code: int | None
) -> None:
    """Test that the entry_point function catches SystemExit and returns the exit code."""

    def raise_exit() -> None:
        raise SystemExit(code)

    monkeypatch.setattr(bijux_cli, "main", raise_exit)

    result = bijux_cli.entry_point()
    expected = int(code or 0)
    assert result == expected


def test_reimport_does_not_raise() -> None:
    """Test that reloading the package does not cause side effects or raise errors."""
    importlib.reload(bijux_cli)
