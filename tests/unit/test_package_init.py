# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for top-level package exports."""

from __future__ import annotations

import pytest

import bijux_cli


def test_entry_point_returns_main_value(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(bijux_cli, "main", lambda: 7)
    assert bijux_cli.entry_point() == 7


def test_entry_point_handles_system_exit(monkeypatch: pytest.MonkeyPatch) -> None:
    def _boom() -> int:
        raise SystemExit(3)

    monkeypatch.setattr(bijux_cli, "main", _boom)
    assert bijux_cli.entry_point() == 3


def test_main_imports_bootstrap(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: dict[str, int] = {"count": 0}

    def _fake_main() -> int:
        calls["count"] += 1
        return 0

    monkeypatch.setattr("bijux_cli.core.bootstrap.main", _fake_main)
    assert bijux_cli.main() == 0
    assert calls["count"] == 1
