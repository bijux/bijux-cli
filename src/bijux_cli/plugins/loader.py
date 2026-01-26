# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Lazy plugin command loading helpers."""

from __future__ import annotations

import importlib.util
import sys
from types import ModuleType
from typing import Any

import click
import typer

from bijux_cli.core.runtime import AsyncTyper, adapt_typer
from bijux_cli.plugins.metadata import (
    PluginMetadata,
    PluginMetadataError,
    validate_plugin_metadata,
)


def _load_module_from_path(path: str, module_name: str) -> ModuleType:
    """Load a module from a file path and register it in sys.modules."""
    spec = importlib.util.spec_from_file_location(module_name, path)
    if not spec or not spec.loader:
        raise PluginMetadataError(f"Cannot import plugin: {path}", http_status=400)
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    try:
        spec.loader.exec_module(module)
    except (FileNotFoundError, OSError) as exc:
        raise PluginMetadataError(
            f"Cannot import plugin: {path}", http_status=400
        ) from exc
    return module


def _load_typer_from_module(module: ModuleType) -> typer.Typer:
    """Extract a Typer app from a plugin module and adapt it for async."""
    if hasattr(module, "cli") and callable(module.cli):
        app = module.cli()
    elif hasattr(module, "app"):
        app = module.app
    else:
        raise PluginMetadataError(
            "Plugin has no CLI entrypoint (expected cli() or app)",
            http_status=400,
        )
    if not isinstance(app, typer.Typer):
        raise PluginMetadataError(
            "Plugin CLI entrypoint did not return a Typer app",
            http_status=400,
        )
    adapt_typer(app)
    return app


def _entrypoint_loader(meta: PluginMetadata) -> typer.Typer:
    """Load a Typer app from entry point metadata."""
    if not meta.entrypoint:
        raise PluginMetadataError("Entry point metadata missing", http_status=400)
    obj = meta.entrypoint.load()
    if isinstance(obj, typer.Typer):
        adapt_typer(obj)
        return obj
    if callable(obj):
        obj = obj()
    if hasattr(obj, "registered_groups"):
        app = typer.Typer()
        for name, sub in getattr(obj, "registered_groups", {}).items():
            if isinstance(sub, typer.Typer):
                adapt_typer(sub)
                app.add_typer(sub, name=name)
        return app
    if hasattr(obj, "register") and callable(obj.register):
        app = typer.Typer()
        obj.register(app)
        adapt_typer(app)
        return app
    if hasattr(obj, "app") and isinstance(obj.app, typer.Typer):
        adapt_typer(obj.app)
        return obj.app
    raise PluginMetadataError(
        f"Entry point {meta.name!r} did not provide a Typer app",
        http_status=400,
    )


def _local_loader(meta: PluginMetadata) -> typer.Typer:
    """Load a local plugin by importing its plugin.py module."""
    if not meta.path:
        raise PluginMetadataError("Local plugin path missing", http_status=400)
    plug_py = meta.path / "plugin.py"
    module = _load_module_from_path(str(plug_py), f"_bijux_cli_plugin_{meta.name}")
    return _load_typer_from_module(module)


class LazyTyper(AsyncTyper):
    """Typer app that loads a plugin Typer app on first access."""

    def __init__(self, meta: PluginMetadata):
        """Initialize a lazy-loading Typer wrapper."""
        super().__init__(name=meta.name, invoke_without_command=True)
        self._meta = meta
        self._loaded: click.Group | None = None

    def _load(self) -> click.Group:
        """Load the underlying plugin command tree on first access."""
        if self._loaded is None:
            if self._meta.source == "entrypoint":
                app = _entrypoint_loader(self._meta)
            else:
                app = _local_loader(self._meta)
            loaded = typer.main.get_command(app)
            if isinstance(loaded, click.Group):
                self._loaded = loaded
            else:
                wrapper = click.Group(name=self._meta.name)
                wrapper.add_command(loaded, loaded.name)
                self._loaded = wrapper
        return self._loaded

    def list_commands(self, ctx: click.Context) -> list[str]:
        """List commands after loading the plugin."""
        return self._load().list_commands(ctx)

    def get_command(self, ctx: click.Context, name: str) -> click.Command | None:
        """Resolve a command after loading the plugin."""
        return self._load().get_command(ctx, name)

    def invoke(self, ctx: click.Context) -> Any:
        """Invoke the loaded plugin command group."""
        return self._load().invoke(ctx)


def lazy_command_for(meta: PluginMetadata) -> typer.Typer:
    """Return a lazy-loading Typer wrapper for a plugin."""
    return load_command_for(meta)


def load_command_for(meta: PluginMetadata) -> typer.Typer:
    """Load a plugin Typer app immediately."""
    # Invariant: metadata validation happens before any activation attempt.
    validate_plugin_metadata(meta)
    if meta.source == "entrypoint":
        return _entrypoint_loader(meta)
    return _local_loader(meta)


def activate_plugin(meta: PluginMetadata) -> typer.Typer:
    """Activate a plugin and return its Typer command tree."""
    return load_command_for(meta)


def deactivate_plugin(_meta: PluginMetadata) -> None:
    """Deactivate a plugin if applicable (no-op for now)."""
    return None
