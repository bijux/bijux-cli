# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Tests for flag precedence and logging semantics."""

from __future__ import annotations

from bijux_cli.core.precedence import resolve_output_flags


def test_resolve_output_flags_default() -> None:
    result = resolve_output_flags(
        quiet=False,
        verbose=False,
        debug=False,
        pretty=False,
        log_level="info",
        color="auto",
    )
    assert result["log_level"] == "info"
    assert result["include_runtime"] is False
    assert result["pretty"] is False
    assert result["color"] == "auto"


def test_resolve_output_flags_verbose() -> None:
    result = resolve_output_flags(
        quiet=False,
        verbose=True,
        debug=False,
        pretty=False,
        log_level="info",
        color="auto",
    )
    assert result["log_level"] == "info"
    assert result["include_runtime"] is True
    assert result["pretty"] is False


def test_resolve_output_flags_debug_overrides() -> None:
    result = resolve_output_flags(
        quiet=False,
        verbose=False,
        debug=True,
        pretty=False,
        log_level="warning",
        color="always",
    )
    assert result["log_level"] == "debug"
    assert result["include_runtime"] is True
    assert result["pretty"] is True
    assert result["color"] == "always"


def test_resolve_output_flags_quiet_wins() -> None:
    result = resolve_output_flags(
        quiet=True,
        verbose=True,
        debug=True,
        pretty=True,
        log_level="debug",
        color="auto",
    )
    assert result["log_level"] == "error"
    assert result["include_runtime"] is False
    assert result["pretty"] is True
