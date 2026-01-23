# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Centralized color mode handling for CLI output."""

from __future__ import annotations

from bijux_cli.core.enums import ColorMode, OutputFormat

_COLOR_MODE = ColorMode.AUTO


def set_color_mode(mode: ColorMode) -> None:
    """Set the global color mode for Click/Typer output."""
    global _COLOR_MODE
    _COLOR_MODE = mode


def get_color_mode() -> ColorMode:
    """Return the current global color mode."""
    return _COLOR_MODE


def resolve_click_color(*, quiet: bool, fmt: OutputFormat | None = None) -> bool | None:
    """Resolve Click/Typer color usage for the current mode and output."""
    if quiet:
        return False
    if fmt in (OutputFormat.JSON, OutputFormat.YAML):
        return False
    if _COLOR_MODE is ColorMode.NEVER:
        return False
    if _COLOR_MODE is ColorMode.ALWAYS:
        return True
    return None


def resolve_color_mode(
    cli: ColorMode | None,
    env: ColorMode | None,
    config: ColorMode | None,
    tty: bool,
) -> ColorMode:
    """Resolve the effective color mode for CLI rendering."""
    for value in (cli, env, config):
        if value is not None:
            mode = value
            break
    else:
        mode = ColorMode.AUTO
    if mode is ColorMode.AUTO and not tty:
        return ColorMode.NEVER
    return mode


__all__ = [
    "get_color_mode",
    "resolve_click_color",
    "resolve_color_mode",
    "set_color_mode",
]
