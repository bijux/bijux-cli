# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Regression coverage for plugin metadata discovery."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import pytest

from bijux_cli.plugins.metadata import (
    PluginMetadataError,
    discover_plugins,
    invalidate_plugin_cache,
)


@dataclass
class _FakeDist:
    name: str
    version: str
    metadata: object


class _FakeMeta:
    def __init__(self, name: str, requires: list[str]) -> None:
        self._name = name
        self._requires = requires

    def get(self, key: str) -> str | None:
        if key == "Name":
            return self._name
        return None

    def get_all(self, key: str) -> list[str]:
        if key == "Requires-Dist":
            return list(self._requires)
        return []


class _FakeEntryPoint:
    def __init__(self, name: str, module: str, dist: _FakeDist | None = None) -> None:
        self.name = name
        self.module = module
        self.dist = dist


class _FakeEntryPoints:
    def __init__(self, entries: list[_FakeEntryPoint]) -> None:
        self._entries = entries

    def select(self, *, group: str) -> list[_FakeEntryPoint]:
        if group == "bijux_cli.plugins":
            return list(self._entries)
        return []


def _write_plugin(tmp_path: Path, name: str, meta: dict[str, object]) -> Path:
    plug_dir = tmp_path / name
    plug_dir.mkdir(parents=True, exist_ok=True)
    (plug_dir / "plugin.py").write_text(
        "import typer\napp = typer.Typer()\n", encoding="utf-8"
    )
    meta = {**meta, "schema_version": meta.get("schema_version", "1")}
    (plug_dir / "plugin.json").write_text(
        __import__("json").dumps(meta), encoding="utf-8"
    )
    return plug_dir


def test_local_missing_required_fields(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(tmp_path, "missing_meta", {"name": "missing_meta"})
    invalidate_plugin_cache()
    with pytest.raises(PluginMetadataError):
        discover_plugins()


def test_local_name_mismatch(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(
        tmp_path,
        "dir_name",
        {"name": "other_name", "version": "0.1.0", "bijux_cli_version": ">=0"},
    )
    invalidate_plugin_cache()
    with pytest.raises(PluginMetadataError):
        discover_plugins()


def test_incompatible_cli_spec(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(
        tmp_path,
        "bad_spec",
        {"name": "bad_spec", "version": "0.1.0", "bijux_cli_version": ">=999.0.0"},
    )
    invalidate_plugin_cache()
    with pytest.raises(PluginMetadataError):
        discover_plugins()


def test_schema_version_required(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(
        tmp_path,
        "no_schema",
        {
            "name": "no_schema",
            "version": "0.1.0",
            "bijux_cli_version": ">=0",
            "schema_version": "",
        },
    )
    invalidate_plugin_cache()
    with pytest.raises(PluginMetadataError):
        discover_plugins()


def test_schema_version_mismatch(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(
        tmp_path,
        "bad_schema",
        {
            "name": "bad_schema",
            "version": "0.1.0",
            "bijux_cli_version": ">=0",
            "schema_version": "2",
        },
    )
    invalidate_plugin_cache()
    with pytest.raises(PluginMetadataError):
        discover_plugins()


def test_entrypoint_missing_cli_requirement(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    meta = _FakeMeta("entrypkg", ["some-lib>=1.0"])
    ep = _FakeEntryPoint(
        "entry_plugin", "entrypkg.mod", _FakeDist("entrypkg", "1.0", meta)
    )
    monkeypatch.setattr(
        "importlib.metadata.entry_points", lambda: _FakeEntryPoints([ep])
    )
    invalidate_plugin_cache()
    with pytest.raises(PluginMetadataError):
        discover_plugins()


def test_entrypoint_duplicate_name(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(
        tmp_path,
        "dupe",
        {"name": "dupe", "version": "0.1.0", "bijux_cli_version": ">=0"},
    )
    meta = _FakeMeta("entrypkg", ["bijux-cli>=0"])
    ep = _FakeEntryPoint("dupe", "entrypkg.mod", _FakeDist("entrypkg", "1.0", meta))
    monkeypatch.setattr(
        "importlib.metadata.entry_points", lambda: _FakeEntryPoints([ep])
    )
    invalidate_plugin_cache()
    with pytest.raises(PluginMetadataError):
        discover_plugins()


def test_unknown_fields_are_ignored(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(
        tmp_path,
        "extra",
        {
            "name": "extra",
            "version": "0.1.0",
            "bijux_cli_version": ">=0",
            "extra_field": "ok",
        },
    )
    invalidate_plugin_cache()
    plugins = discover_plugins()
    assert [p.name for p in plugins] == ["extra"]


def test_discovery_order_is_sorted(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(
        tmp_path,
        "zeta",
        {"name": "zeta", "version": "0.1.0", "bijux_cli_version": ">=0"},
    )
    _write_plugin(
        tmp_path,
        "alpha",
        {"name": "alpha", "version": "0.1.0", "bijux_cli_version": ">=0"},
    )
    invalidate_plugin_cache()
    plugins = discover_plugins()
    assert [p.name for p in plugins] == ["alpha", "zeta"]


def test_cache_invalidation(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path))
    _write_plugin(
        tmp_path,
        "cache_a",
        {"name": "cache_a", "version": "0.1.0", "bijux_cli_version": ">=0"},
    )
    invalidate_plugin_cache()
    plugins = discover_plugins()
    assert [p.name for p in plugins] == ["cache_a"]

    _write_plugin(
        tmp_path,
        "cache_b",
        {"name": "cache_b", "version": "0.1.0", "bijux_cli_version": ">=0"},
    )
    cached = discover_plugins()
    assert [p.name for p in cached] == ["cache_a"]

    invalidate_plugin_cache()
    refreshed = discover_plugins()
    assert sorted(p.name for p in refreshed) == ["cache_a", "cache_b"]
