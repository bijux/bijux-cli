# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for plugin catalog helpers."""

from __future__ import annotations

import pytest

from bijux_cli.plugins import catalog


def test_list_installed_plugins_delegates(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.list_plugins", lambda: [{"name": "plug"}]
    )
    assert catalog.list_installed_plugins() == [{"name": "plug"}]


def test_invalidate_cache_delegates(monkeypatch: pytest.MonkeyPatch) -> None:
    called: dict[str, int] = {"count": 0}

    def _invalidate() -> None:
        called["count"] += 1

    monkeypatch.setattr(
        "bijux_cli.plugins.metadata.invalidate_plugin_cache", _invalidate
    )
    catalog.invalidate_cache()
    assert called["count"] == 1
