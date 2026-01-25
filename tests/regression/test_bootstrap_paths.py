# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Regression coverage for bootstrap fast vs runtime paths."""

from __future__ import annotations

from pathlib import Path

import pytest

from bijux_cli.core import bootstrap_impl


def _isolate_env(monkeypatch: pytest.MonkeyPatch, root: Path) -> None:
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(root / ".env"))
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(root / "plugins"))
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(root / ".history"))
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "1")


def test_fast_version_skips_di_init(monkeypatch: pytest.MonkeyPatch) -> None:
    called = {"default": 0, "plugins": 0, "engine": 0}

    def _mark_default(*_args: object, **_kwargs: object) -> None:
        called["default"] += 1

    def _mark_plugins(*_args: object, **_kwargs: object) -> None:
        called["plugins"] += 1

    def _mark_engine(*_args: object, **_kwargs: object) -> None:
        called["engine"] += 1

    monkeypatch.setattr(bootstrap_impl, "register_default_services", _mark_default)
    monkeypatch.setattr(bootstrap_impl, "register_plugin_services", _mark_plugins)
    monkeypatch.setattr(bootstrap_impl, "Engine", _mark_engine)
    monkeypatch.setattr(
        bootstrap_impl.sys,
        "argv",
        ["bijux", "version"],
    )

    exit_code = bootstrap_impl.main()
    assert exit_code == 0
    assert called == {"default": 0, "plugins": 0, "engine": 0}


def test_runtime_path_initializes_di(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    called = {"default": 0, "plugins": 0, "engine": 0}
    original_default = bootstrap_impl.register_default_services
    original_plugins = bootstrap_impl.register_plugin_services
    original_engine = bootstrap_impl.Engine

    def _mark_default(*_args: object, **_kwargs: object) -> None:
        called["default"] += 1
        original_default(*_args, **_kwargs)

    def _mark_plugins(*_args: object, **_kwargs: object) -> None:
        called["plugins"] += 1
        original_plugins(*_args, **_kwargs)

    def _mark_engine(*_args: object, **_kwargs: object) -> None:
        called["engine"] += 1
        original_engine(*_args, **_kwargs)

    _isolate_env(monkeypatch, tmp_path)
    monkeypatch.setenv("HOME", str(tmp_path / "home"))
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "xdg_config"))
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path / "xdg_cache"))
    monkeypatch.setenv("XDG_DATA_HOME", str(tmp_path / "xdg_data"))

    monkeypatch.setattr(
        bootstrap_impl.sys,
        "argv",
        ["bijux", "status"],
    )

    # Bind wrappers with access to originals.
    monkeypatch.setattr(bootstrap_impl, "register_default_services", _mark_default)
    monkeypatch.setattr(bootstrap_impl, "register_plugin_services", _mark_plugins)
    monkeypatch.setattr(bootstrap_impl, "Engine", _mark_engine)

    exit_code = bootstrap_impl.main()
    assert exit_code in (0, 1, 2)
    assert called["default"] >= 1
    assert called["plugins"] >= 1
    assert called["engine"] >= 1
