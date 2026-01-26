# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Regression coverage for plugin loader paths."""

from __future__ import annotations

import importlib.metadata as im
from pathlib import Path
from typing import cast

import pytest
import typer

from bijux_cli.plugins.loader import (
    activate_plugin,
    deactivate_plugin,
    load_command_for,
)
from bijux_cli.plugins.metadata import PluginMetadata, PluginMetadataError


def _write_local_plugin(path: Path, name: str) -> None:
    path.mkdir(parents=True, exist_ok=True)
    (path / "plugin.py").write_text(
        "\n".join(
            [
                "import typer",
                "app = typer.Typer()",
                "",
                "@app.command()",
                "def hello() -> None:",
                '    typer.echo("hi")',
            ]
        ),
        encoding="utf-8",
    )


def test_local_module_loader(tmp_path: Path) -> None:
    plugin_dir = tmp_path / "local_plugin"
    _write_local_plugin(plugin_dir, "local_plugin")
    meta = PluginMetadata(
        name="local_plugin",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0",
        path=plugin_dir,
    )
    app = load_command_for(meta)
    assert isinstance(app, typer.Typer)


def test_entrypoint_loader() -> None:
    entry_app = typer.Typer()

    class _FakeEntryPoint:
        def load(self) -> typer.Typer:
            return entry_app

    meta = PluginMetadata(
        name="entry_plugin",
        version="0.1.0",
        enabled=True,
        source="entrypoint",
        requires_cli=">=0",
        entrypoint=cast(im.EntryPoint, _FakeEntryPoint()),
    )
    app = load_command_for(meta)
    assert isinstance(app, typer.Typer)


def test_activation_and_deactivation(tmp_path: Path) -> None:
    plugin_dir = tmp_path / "active_plugin"
    _write_local_plugin(plugin_dir, "active_plugin")
    meta = PluginMetadata(
        name="active_plugin",
        version="0.1.0",
        enabled=True,
        source="local",
        requires_cli=">=0",
        path=plugin_dir,
    )
    app = activate_plugin(meta)
    assert isinstance(app, typer.Typer)
    deactivate_plugin(meta)


def test_entrypoint_missing_metadata() -> None:
    meta = PluginMetadata(
        name="broken_entry",
        version="0.1.0",
        enabled=True,
        source="entrypoint",
        requires_cli=">=0",
        entrypoint=None,
    )
    with pytest.raises(PluginMetadataError):
        load_command_for(meta)
