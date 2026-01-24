# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for CLI color handling."""

from __future__ import annotations

from bijux_cli.cli.color import (
    get_color_mode,
    resolve_click_color,
    resolve_color_mode,
    set_color_mode,
)
from bijux_cli.core.enums import ColorMode, OutputFormat
from bijux_cli.core.precedence import FlagLayer, GlobalCLIConfig


def _config(color: ColorMode | None) -> GlobalCLIConfig:
    return GlobalCLIConfig(
        help=False,
        flags=FlagLayer(color=color),
        args=(),
        errors=(),
    )


def test_resolve_color_mode_no_color_overrides() -> None:
    assert resolve_color_mode(_config(ColorMode.ALWAYS), tty=True, no_color=True) is (
        ColorMode.NEVER
    )


def test_resolve_color_mode_auto_non_tty() -> None:
    assert (
        resolve_color_mode(_config(None), tty=False, no_color=False) is ColorMode.NEVER
    )


def test_resolve_color_mode_auto_tty() -> None:
    assert resolve_color_mode(_config(None), tty=True, no_color=False) is ColorMode.AUTO


def test_resolve_click_color_rules() -> None:
    set_color_mode(ColorMode.AUTO)
    assert resolve_click_color(quiet=True, fmt=None) is False
    assert resolve_click_color(quiet=False, fmt=OutputFormat.JSON) is False
    assert resolve_click_color(quiet=False, fmt=OutputFormat.YAML) is False

    set_color_mode(ColorMode.NEVER)
    assert resolve_click_color(quiet=False, fmt=None) is False

    set_color_mode(ColorMode.ALWAYS)
    assert resolve_click_color(quiet=False, fmt=None) is True

    set_color_mode(ColorMode.AUTO)
    assert resolve_click_color(quiet=False, fmt=None) is None
    assert get_color_mode() is ColorMode.AUTO


def test_resolve_color_mode_explicit_flags() -> None:
    assert resolve_color_mode(_config(ColorMode.ALWAYS), tty=True, no_color=False) is (
        ColorMode.ALWAYS
    )
    assert resolve_color_mode(_config(ColorMode.NEVER), tty=True, no_color=False) is (
        ColorMode.NEVER
    )
    assert resolve_color_mode(_config(ColorMode.AUTO), tty=True, no_color=False) is (
        ColorMode.AUTO
    )


def test_machine_output_never_styled() -> None:
    set_color_mode(ColorMode.ALWAYS)
    assert resolve_click_color(quiet=False, fmt=OutputFormat.JSON) is False
    assert resolve_click_color(quiet=False, fmt=OutputFormat.YAML) is False
