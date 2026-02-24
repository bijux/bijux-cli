# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

from __future__ import annotations

from types import SimpleNamespace

import pytest

from bijux_cli.cli.commands.atlas.service import atlas
from bijux_cli.cli.commands.dev.atlas.service import dev_atlas


def test_atlas_routes_args(monkeypatch: pytest.MonkeyPatch) -> None:
    captured: dict[str, object] = {}

    def _fake_run_external(binary: object, args: list[str]) -> int:
        captured["binary"] = binary
        captured["args"] = args
        return 7

    monkeypatch.setattr(
        "bijux_cli.cli.commands.atlas.service.run_external",
        _fake_run_external,
    )

    ctx = SimpleNamespace(invoked_subcommand=None, args=["check", "run"])
    with pytest.raises(SystemExit) as exc:
        atlas(ctx)  # type: ignore[arg-type]
    assert exc.value.code == 7
    assert captured["args"] == ["check", "run"]


def test_dev_atlas_routes_args(monkeypatch: pytest.MonkeyPatch) -> None:
    captured: dict[str, object] = {}

    def _fake_run_external(binary: object, args: list[str]) -> int:
        captured["binary"] = binary
        captured["args"] = args
        return 9

    monkeypatch.setattr(
        "bijux_cli.cli.commands.dev.atlas.service.run_external",
        _fake_run_external,
    )

    ctx = SimpleNamespace(invoked_subcommand=None, args=["doctor"])
    with pytest.raises(SystemExit) as exc:
        dev_atlas(ctx)  # type: ignore[arg-type]
    assert exc.value.code == 9
    assert captured["args"] == ["doctor"]


def test_atlas_legacy_dev_route_redirects(monkeypatch: pytest.MonkeyPatch) -> None:
    def _unexpected_run_external(binary: object, args: list[str]) -> int:
        raise AssertionError("run_external must not be called for legacy redirect")

    monkeypatch.setattr(
        "bijux_cli.cli.commands.atlas.service.run_external",
        _unexpected_run_external,
    )

    ctx = SimpleNamespace(invoked_subcommand=None, args=["dev-atlas", "doctor"])
    with pytest.raises(SystemExit) as exc:
        atlas(ctx)  # type: ignore[arg-type]
    assert exc.value.code == 2
