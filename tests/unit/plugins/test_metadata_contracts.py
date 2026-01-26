# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Plugin metadata contract tests."""

from __future__ import annotations

from importlib.metadata import EntryPoint
import json
from pathlib import Path
from typing import cast

import pytest

import bijux_cli.plugins.metadata as metadata_mod
from bijux_cli.plugins.metadata import (
    PluginMetadata,
    PluginMetadataError,
    discover_plugins,
    invalidate_plugin_cache,
    validate_plugin_metadata,
)


def _write_plugin(dir_path: Path, payload: dict[str, object]) -> None:
    dir_path.mkdir(parents=True, exist_ok=True)
    payload = {**payload, "schema_version": payload.get("schema_version", "1")}
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


def test_validate_plugin_metadata_rejects_schema_version() -> None:
    meta = PluginMetadata(
        name="demo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0",
        schema_version="2",
    )
    with pytest.raises(PluginMetadataError):
        validate_plugin_metadata(meta)


def test_validate_plugin_metadata_rejects_invalid_spec() -> None:
    meta = PluginMetadata(
        name="demo",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli="not-a-spec",
        schema_version="1",
    )
    with pytest.raises(PluginMetadataError):
        validate_plugin_metadata(meta)


def test_plugin_meta_from_dist_rejects_invalid_name() -> None:
    class _EP:
        name = "bad name"
        module = "badmod"
        dist = None

    with pytest.raises(PluginMetadataError):
        metadata_mod._plugin_meta_from_dist(cast(EntryPoint, _EP()))


def test_plugin_meta_from_dist_missing_requirement(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class _Meta:
        def get(self, key: str) -> str | None:
            return "demo" if key == "Name" else None

        def get_all(self, key: str) -> list[str]:
            return ["other>=1.0"] if key == "Requires-Dist" else []

    class _Dist:
        name = "demo"
        version = "1.0.0"
        metadata = _Meta()

    class _EP:
        name = "demo"
        module = "demo.mod"
        dist = _Dist()

    with pytest.raises(PluginMetadataError):
        metadata_mod._plugin_meta_from_dist(cast(EntryPoint, _EP()))


def test_plugin_meta_from_local_invalid_json(tmp_path: Path) -> None:
    plug = tmp_path / "bad"
    plug.mkdir()
    (plug / "plugin.py").write_text("app=None\n", encoding="utf-8")
    (plug / "plugin.json").write_text("{bad json", encoding="utf-8")
    with pytest.raises(PluginMetadataError):
        metadata_mod._plugin_meta_from_local(plug)
