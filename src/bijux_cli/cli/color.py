# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Centralized color mode handling for CLI output."""

from __future__ import annotations

from bijux_cli.core.enums import ColorMode

_COLOR_MODE = ColorMode.AUTO


def set_color_mode(mode: ColorMode) -> None:
    """Set the global color mode for Click/Typer output."""
    global _COLOR_MODE
    _COLOR_MODE = mode


def get_color_mode() -> ColorMode:
    """Return the current global color mode."""
    return _COLOR_MODE


def apply_color_mode(color: bool | None) -> bool | None:
    """Apply the current color mode to a Click/Typer color flag."""
    if _COLOR_MODE is ColorMode.NEVER:
        return False
    if _COLOR_MODE is ColorMode.ALWAYS:
        return True
    return color


__all__ = ["apply_color_mode", "get_color_mode", "set_color_mode"]
