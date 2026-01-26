# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Extra coverage for plugin loader helpers."""

from __future__ import annotations

import importlib.abc
import importlib.machinery
from pathlib import Path
from types import ModuleType, SimpleNamespace
from typing import Any, cast

import click
import pytest
import typer

from bijux_cli.plugins.loader import (
    LazyTyper,
    _entrypoint_loader,
    _load_module_from_path,
    _load_typer_from_module,
    _local_loader,
)
from bijux_cli.plugins.metadata import PluginMetadata, PluginMetadataError


def _meta(**kwargs: object) -> PluginMetadata:
    name = cast(str, kwargs.pop("name", "demo"))
    version = cast(str, kwargs.pop("version", "0.1.0"))
    enabled = cast(bool, kwargs.pop("enabled", True))
    source = cast(str, kwargs.pop("source", "entrypoint"))
    requires_cli = cast(str, kwargs.pop("requires_cli", ">=0"))
    schema_version = cast(str, kwargs.pop("schema_version", "1"))
    entrypoint = kwargs.pop("entrypoint", None)
    path = cast(Path | None, kwargs.pop("path", None))
    if kwargs:
        raise AssertionError(f"Unexpected metadata keys: {sorted(kwargs)}")
    return PluginMetadata(
        name=name,
        version=version,
        enabled=enabled,
        source=source,
        requires_cli=requires_cli,
        schema_version=schema_version,
        entrypoint=cast(Any, entrypoint),
        path=path,
    )


def test_load_module_from_path_missing_spec(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        "bijux_cli.plugins.loader.importlib.util.spec_from_file_location",
        lambda *_a, **_k: None,
    )
    with pytest.raises(PluginMetadataError):
        _load_module_from_path("missing.py", "mod")


def test_load_module_from_path_missing_loader(monkeypatch: pytest.MonkeyPatch) -> None:
    class _Spec:
        loader = None

    monkeypatch.setattr(
        "bijux_cli.plugins.loader.importlib.util.spec_from_file_location",
        lambda *_a, **_k: _Spec(),
    )
    with pytest.raises(PluginMetadataError):
        _load_module_from_path("missing.py", "mod")


def test_load_module_from_path_file_not_found(monkeypatch: pytest.MonkeyPatch) -> None:
    class _Loader(importlib.abc.Loader):
        def create_module(self, _spec: importlib.machinery.ModuleSpec) -> None:
            return None

        def exec_module(self, _mod: ModuleType) -> None:
            raise FileNotFoundError("missing")

    spec = importlib.machinery.ModuleSpec(name="mod", loader=_Loader())
    monkeypatch.setattr(
        "bijux_cli.plugins.loader.importlib.util.spec_from_file_location",
        lambda *_a, **_k: spec,
    )
    with pytest.raises(PluginMetadataError):
        _load_module_from_path("missing.py", "mod")


def test_load_typer_from_module_missing_entrypoint() -> None:
    module = ModuleType("mod")
    with pytest.raises(PluginMetadataError):
        _load_typer_from_module(module)


def test_load_typer_from_module_invalid_app() -> None:
    module = ModuleType("mod")
    mod_any = cast(Any, module)
    mod_any.app = object()
    with pytest.raises(PluginMetadataError):
        _load_typer_from_module(module)


def test_entrypoint_loader_missing_entrypoint() -> None:
    meta = _meta(entrypoint=None)
    with pytest.raises(PluginMetadataError):
        _entrypoint_loader(meta)


def test_entrypoint_loader_registered_groups() -> None:
    sub = typer.Typer()

    class _Obj:
        registered_groups = {"sub": sub}

    meta = _meta(entrypoint=SimpleNamespace(load=lambda: _Obj()))
    app = _entrypoint_loader(meta)
    assert isinstance(app, typer.Typer)


def test_entrypoint_loader_returns_typer() -> None:
    app = typer.Typer()
    meta = _meta(entrypoint=SimpleNamespace(load=lambda: app))
    loaded = _entrypoint_loader(meta)
    assert loaded is app


def test_entrypoint_loader_callable_app() -> None:
    class _Obj:
        def register(self, app: typer.Typer) -> None:
            @app.command()
            def hello() -> None:
                return None

    meta = _meta(entrypoint=SimpleNamespace(load=lambda: (lambda: _Obj())))
    app = _entrypoint_loader(meta)
    assert isinstance(app, typer.Typer)


def test_entrypoint_loader_register_method() -> None:
    class _Obj:
        def register(self, app: typer.Typer) -> None:
            @app.command()
            def ping() -> None:
                return None

    meta = _meta(entrypoint=SimpleNamespace(load=lambda: _Obj()))
    app = _entrypoint_loader(meta)
    assert isinstance(app, typer.Typer)


def test_entrypoint_loader_app_attr() -> None:
    obj = SimpleNamespace(app=typer.Typer())
    meta = _meta(entrypoint=SimpleNamespace(load=lambda: obj))
    app = _entrypoint_loader(meta)
    assert app is obj.app


def test_local_loader_missing_path() -> None:
    meta = _meta(source="local", path=None)
    with pytest.raises(PluginMetadataError):
        _local_loader(meta)


def test_load_typer_from_module_cli_callable() -> None:
    module = ModuleType("mod")

    def cli() -> typer.Typer:
        return typer.Typer()

    mod_any = cast(Any, module)
    mod_any.cli = cli
    app = _load_typer_from_module(module)
    assert isinstance(app, typer.Typer)


def test_load_module_from_path_success(tmp_path: Path) -> None:
    plugin_file = tmp_path / "plugin.py"
    plugin_file.write_text(
        "import typer\n\ndef cli():\n    app = typer.Typer()\n    return app\n",
        encoding="utf-8",
    )
    module = _load_module_from_path(str(plugin_file), "demo_mod")
    assert hasattr(module, "cli")


def test_local_loader_success(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    (plug_dir / "plugin.py").write_text(
        "import typer\n\ndef cli():\n    app = typer.Typer()\n    return app\n",
        encoding="utf-8",
    )
    meta = _meta(source="local", path=plug_dir)
    app = _local_loader(meta)
    assert isinstance(app, typer.Typer)


def test_lazy_typer_loads_command(tmp_path: Path) -> None:
    plug_dir = tmp_path / "demo"
    plug_dir.mkdir()
    (plug_dir / "plugin.py").write_text(
        "import typer\n\n"
        "app = typer.Typer()\n"
        "@app.command()\n"
        "def hello():\n"
        "    return None\n",
        encoding="utf-8",
    )
    meta = _meta(source="local", path=plug_dir)
    lazy = LazyTyper(meta)
    group = lazy._load()
    commands = group.list_commands(click.Context(group))
    assert "hello" in commands
