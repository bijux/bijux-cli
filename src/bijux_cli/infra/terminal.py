# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Terminal adapter interfaces and no-op defaults."""

from __future__ import annotations

class NoopTerminal:
    """No-op terminal adapter that disables styling."""

    def supports_color(self) -> bool:
        return False

    def style(self, text: str, *, color: str | None = None) -> str:
        return text


__all__ = ["NoopTerminal"]
