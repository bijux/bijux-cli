# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI root cli module."""

from __future__ import annotations

import subprocess
import sys
from typing import Any

import pytest
import typer

import bijux_cli.cli as cli_mod


class DummyCtx:
    """A minimal mock for Typer's Context."""

    def __init__(self, invoked_subcommand: str | None) -> None:
        """Initialize the dummy context."""
        self.invoked_subcommand = invoked_subcommand


def test_maybe_default_to_repl_invokes_subprocess_when_no_subcommand(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that the REPL is invoked when no subcommand is given."""
    called: dict[str, Any] = {}

    def fake_call(args: list[str]) -> int:
        called["args"] = args
        return 123

    monkeypatch.setattr(subprocess, "call", fake_call)

    ctx = DummyCtx(invoked_subcommand=None)
    cli_mod.maybe_default_to_repl(ctx)  # type: ignore[arg-type]

    assert "args" in called
    assert called["args"] == [sys.argv[0], "repl"]


def test_maybe_default_to_repl_skips_when_there_is_a_subcommand(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that the REPL is not invoked when a subcommand is present."""
    monkeypatch.setattr(
        subprocess,
        "call",
        lambda args: (_ for _ in ()).throw(AssertionError("should not be called")),
    )

    ctx = DummyCtx(invoked_subcommand="foo")
    cli_mod.maybe_default_to_repl(ctx)  # type: ignore[arg-type]


def test_module_level_app_is_build_app() -> None:
    """Test that the module-level app is a Typer instance and is distinct from new builds."""
    assert isinstance(cli_mod.app, typer.Typer)

    new_app = cli_mod.build_app()
    assert new_app is not cli_mod.app

    assert new_app.info.help == cli_mod.app.info.help
