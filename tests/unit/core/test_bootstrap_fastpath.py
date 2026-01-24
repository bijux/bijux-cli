# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

from __future__ import annotations

import importlib.metadata as importlib_metadata
import sys

import pytest

from bijux_cli.core import bootstrap
from bijux_cli.core.di import DIContainer


def _forbid_di() -> None:
    raise AssertionError("DIContainer.current should not run")


def _forbid_plugins(*_args: object, **_kwargs: object) -> None:
    raise AssertionError("register_plugin_services should not run")


def _forbid_engine(*_args: object, **_kwargs: object) -> None:
    raise AssertionError("Engine should not run")


def test_split_command_args_skips_flags() -> None:
    command, rest = bootstrap._split_command_args(
        ["--quiet", "--format", "json", "version", "--log-level", "debug"]
    )
    assert command == "version"
    assert rest == ["--log-level", "debug"]


def test_fast_help_skips_di(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr(DIContainer, "current", staticmethod(_forbid_di))
    monkeypatch.setattr(
        "bijux_cli.plugins.services.register_plugin_services", _forbid_plugins
    )
    monkeypatch.setattr("bijux_cli.core.engine.Engine", _forbid_engine)
    monkeypatch.setattr(sys, "argv", ["bijux", "--help"])

    exit_code = bootstrap.main()
    assert exit_code == 0
    assert capsys.readouterr().out


def test_fast_version_flag_skips_di(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr(DIContainer, "current", staticmethod(_forbid_di))
    monkeypatch.setattr(
        "bijux_cli.plugins.services.register_plugin_services", _forbid_plugins
    )
    monkeypatch.setattr("bijux_cli.core.engine.Engine", _forbid_engine)
    monkeypatch.setattr(importlib_metadata, "version", lambda _n: "9.9.9")
    monkeypatch.setattr(sys, "argv", ["bijux", "--version"])

    exit_code = bootstrap.main()
    assert exit_code == 0
    assert '"version": "9.9.9"' in capsys.readouterr().out


def test_fast_version_command_skips_di(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr(DIContainer, "current", staticmethod(_forbid_di))
    monkeypatch.setattr(
        "bijux_cli.plugins.services.register_plugin_services", _forbid_plugins
    )
    monkeypatch.setattr("bijux_cli.core.engine.Engine", _forbid_engine)
    monkeypatch.setattr(sys, "argv", ["bijux", "version"])

    exit_code = bootstrap.main()
    assert exit_code == 0
    assert "version" in capsys.readouterr().out
