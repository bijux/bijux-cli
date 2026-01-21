# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for terminal styling adapters."""

from __future__ import annotations

from typing import Protocol, runtime_checkable


@runtime_checkable
class TerminalProtocol(Protocol):
    """Defines the contract for terminal styling."""

    def supports_color(self) -> bool:
        """Return True if the terminal supports color."""
        ...

    def style(self, text: str, *, color: str | None) -> str:
        """Apply color styling to text."""
        ...
