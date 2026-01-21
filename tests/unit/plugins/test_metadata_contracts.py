# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Plugin metadata contract tests."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from bijux_cli.plugins.metadata import (
    PluginMetadataError,
    discover_plugins,
    invalidate_plugin_cache,
)


def _write_plugin(dir_path: Path, payload: dict[str, object]) -> None:
    dir_path.mkdir(parents=True, exist_ok=True)
    (dir_path / "plugin.py").write_text("app = None\n", encoding="utf-8")
    (dir_path / "plugin.json").write_text(json.dumps(payload), encoding="utf-8")


def test_discover_plugins_requires_metadata(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    plugins_dir = tmp_path / "plugins"
    plugins_dir.mkdir()
    (plugins_dir / "broken").mkdir()
    (plugins_dir / "broken" / "plugin.py").write_text("app = None\n", encoding="utf-8")
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(plugins_dir))
    invalidate_plugin_cache()

    with pytest.raises(PluginMetadataError):
        discover_plugins(strict=True)


def test_discover_plugins_rejects_missing_fields(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    plugins_dir = tmp_path / "plugins"
    _write_plugin(plugins_dir / "bad", {"name": "bad", "version": "0.1.0"})
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(plugins_dir))
    invalidate_plugin_cache()

    with pytest.raises(PluginMetadataError):
        discover_plugins(strict=True)


def test_discover_plugins_rejects_name_mismatch(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    plugins_dir = tmp_path / "plugins"
    _write_plugin(
        plugins_dir / "wrongdir",
        {"name": "other", "version": "0.1.0", "bijux_cli_version": ">=0.0.0"},
    )
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(plugins_dir))
    invalidate_plugin_cache()

    with pytest.raises(PluginMetadataError):
        discover_plugins(strict=True)
