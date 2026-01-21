# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Compatibility re-export for async runtime helpers."""

from __future__ import annotations

from importlib import import_module
from typing import Any

__all__ = [
    "AsyncTyper",
    "adapt_typer",
    "command_adapter",
    "run_awaitable",
    "run_command",
]


def _runtime() -> Any:
    return import_module("bijux_cli.app.runtime")


def __getattr__(name: str) -> Any:
    if name in __all__:
        return getattr(_runtime(), name)
    raise AttributeError(name)
