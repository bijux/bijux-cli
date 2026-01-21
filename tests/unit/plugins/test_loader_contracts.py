# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Plugin loader contract tests."""

from __future__ import annotations

from typing import Any, cast

import pytest

from bijux_cli.plugins.loader import activate_plugin
from bijux_cli.plugins.metadata import PluginMetadata, PluginMetadataError


class _BadEntryPoint:
    name = "bad"

    def load(self) -> object:
        return object()


def test_activate_plugin_requires_typer_app() -> None:
    meta = PluginMetadata(
        name="bad",
        version="0.1.0",
        enabled=True,
        source="entrypoint",
        requires_cli=">=0.0.0",
        entrypoint=cast(Any, _BadEntryPoint()),
    )

    with pytest.raises(PluginMetadataError):
        activate_plugin(meta)
