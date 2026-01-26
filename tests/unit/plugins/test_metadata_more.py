# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Extra coverage for plugin metadata helpers."""

from __future__ import annotations

import importlib.metadata as im
import json
from pathlib import Path

import pytest

from bijux_cli.plugins import metadata as meta


def test_require_cli_spec_invalid() -> None:
    with pytest.raises(meta.PluginMetadataError, match="invalid version spec"):
        meta._require_cli_spec("not-a-spec(", name="demo")


def test_require_cli_spec_unsatisfied() -> None:
    with pytest.raises(meta.PluginMetadataError, match="requires bijux-cli"):
        meta._require_cli_spec("<0.0", name="demo")


def test_plugin_meta_from_dist_invalid_name() -> None:
    ep = im.EntryPoint(name="bad name", value="x:y", group="bijux_cli.plugins")
    with pytest.raises(meta.PluginMetadataError, match="invalid"):
        meta._plugin_meta_from_dist(ep)


def test_plugin_meta_from_dist_missing_dist(monkeypatch: pytest.MonkeyPatch) -> None:
    ep = im.EntryPoint(name="demo", value="demo:cli", group="bijux_cli.plugins")
    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.im.distribution",
        lambda _n: (_ for _ in ()).throw(RuntimeError("boom")),
    )
    with pytest.raises(meta.PluginMetadataError, match="no distribution metadata"):
        meta._plugin_meta_from_dist(ep)


def test_plugin_meta_from_dist_missing_requirement(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    ep = im.EntryPoint(name="demo", value="demo:cli", group="bijux_cli.plugins")

    class _Dist:
        version = "1.0"
        name = "demo"

        class Metadata:
            @staticmethod
            def get_all(_key: str) -> list[str]:
                return []

        metadata = Metadata()

    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.im.distribution", lambda _n: _Dist()
    )
    with pytest.raises(meta.PluginMetadataError, match="missing bijux-cli requirement"):
        meta._plugin_meta_from_dist(ep)


def test_plugin_meta_from_local_missing_plugin_json(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    with pytest.raises(meta.PluginMetadataError, match="missing plugin.json"):
        meta._plugin_meta_from_local(plug_dir)


def test_plugin_meta_from_local_invalid_json(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    (plug_dir / "plugin.json").write_text("{bad json}", encoding="utf-8")
    with pytest.raises(meta.PluginMetadataError, match="invalid plugin.json"):
        meta._plugin_meta_from_local(plug_dir)


def test_plugin_meta_from_local_missing_fields(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    (plug_dir / "plugin.json").write_text(
        json.dumps({"name": "demo"}), encoding="utf-8"
    )
    with pytest.raises(meta.PluginMetadataError, match="missing required metadata"):
        meta._plugin_meta_from_local(plug_dir)


def test_plugin_meta_from_local_invalid_schema(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    payload = {
        "name": "demo",
        "version": "1.0",
        "bijux_cli_version": ">=0",
        "schema_version": "2",
    }
    (plug_dir / "plugin.json").write_text(json.dumps(payload), encoding="utf-8")
    with pytest.raises(meta.PluginMetadataError, match="unsupported schema version"):
        meta._plugin_meta_from_local(plug_dir)


def test_plugin_meta_from_local_invalid_name(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    payload = {
        "name": "bad name",
        "version": "1.0",
        "bijux_cli_version": ">=0",
        "schema_version": "1",
    }
    (plug_dir / "plugin.json").write_text(json.dumps(payload), encoding="utf-8")
    with pytest.raises(meta.PluginMetadataError, match="invalid"):
        meta._plugin_meta_from_local(plug_dir)


def test_plugin_meta_from_local_name_mismatch(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    payload = {
        "name": "other",
        "version": "1.0",
        "bijux_cli_version": ">=0",
        "schema_version": "1",
    }
    (plug_dir / "plugin.json").write_text(json.dumps(payload), encoding="utf-8")
    with pytest.raises(meta.PluginMetadataError, match="does not match metadata name"):
        meta._plugin_meta_from_local(plug_dir)


def test_validate_plugin_metadata_errors() -> None:
    base = meta.PluginMetadata(
        name="demo",
        version="1.0",
        enabled=True,
        source="local",
        requires_cli=">=0",
        schema_version="1",
        path=Path("."),
    )
    with pytest.raises(meta.PluginMetadataError, match="missing version"):
        meta.validate_plugin_metadata(
            base.__class__(**{**base.__dict__, "version": ""})
        )
    with pytest.raises(meta.PluginMetadataError, match="missing bijux-cli requirement"):
        meta.validate_plugin_metadata(
            base.__class__(**{**base.__dict__, "requires_cli": ""})
        )
    with pytest.raises(meta.PluginMetadataError, match="missing schema version"):
        meta.validate_plugin_metadata(
            base.__class__(**{**base.__dict__, "schema_version": ""})
        )
    with pytest.raises(meta.PluginMetadataError, match="unsupported schema version"):
        meta.validate_plugin_metadata(
            base.__class__(**{**base.__dict__, "schema_version": "2"})
        )


def test_get_plugin_metadata_not_found(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(meta, "discover_plugins", lambda *a, **k: [])
    with pytest.raises(meta.PluginMetadataError, match="not found"):
        meta.get_plugin_metadata("missing")


def test_plugin_meta_from_local_success(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    payload = {
        "name": "demo",
        "version": "1.0",
        "bijux_cli_version": ">=0",
        "schema_version": "1",
    }
    (plug_dir / "plugin.json").write_text(json.dumps(payload), encoding="utf-8")
    (plug_dir / "plugin.py").write_text("# plugin\n", encoding="utf-8")
    meta_obj = meta._plugin_meta_from_local(plug_dir)
    assert meta_obj.name == "demo"


def test_discover_plugins_entrypoint_and_local(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    class _Dist:
        version = "1.0"
        name = "demo"

        class Metadata:
            @staticmethod
            def get_all(_key: str) -> list[str]:
                return ["bijux-cli>=0"]

        metadata = Metadata()

    class _Entry:
        def __init__(self, name: str) -> None:
            self.name = name
            self.module = name
            self.dist = _Dist()

    class _EntryPoints:
        def select(self, **_kwargs: object) -> list[object]:
            return [_Entry("epdemo")]

    plug_dir = tmp_path / "localdemo"
    plug_dir.mkdir()
    (plug_dir / "plugin.json").write_text(
        json.dumps(
            {
                "name": "localdemo",
                "version": "1.0",
                "bijux_cli_version": ">=0",
                "schema_version": "1",
            }
        ),
        encoding="utf-8",
    )
    (plug_dir / "plugin.py").write_text("# plugin\n", encoding="utf-8")

    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.im.entry_points",
        lambda: _EntryPoints(),
    )
    monkeypatch.setattr(meta, "get_plugins_dir", lambda: tmp_path)
    meta.invalidate_plugin_cache()
    plugins = meta.discover_plugins()
    names = [p.name for p in plugins]
    assert names == ["epdemo", "localdemo"]


def test_list_plugins_and_plugins_for_package(monkeypatch: pytest.MonkeyPatch) -> None:
    plugin = meta.PluginMetadata(
        name="demo",
        version="1.0",
        enabled=True,
        source="entrypoint",
        requires_cli=">=0",
        schema_version="1",
        dist_name="Demo-Pkg",
    )
    monkeypatch.setattr(meta, "discover_plugins", lambda *a, **k: [plugin])
    listed = meta.list_plugins()
    assert listed == [{"name": "demo", "version": "1.0", "enabled": True}]
    assert meta.plugins_for_package("demo-pkg") == [plugin]
