# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the history service."""

from __future__ import annotations

from collections.abc import Sequence
from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class HistoryProtocol(Protocol):
    """Defines the contract for the history service."""

    def add(
        self,
        command: str,
        *,
        params: Sequence[str] | None = None,
        success: bool | None = None,
        return_code: int | None = None,
        duration_ms: float | None = None,
        raw: dict[str, Any] | None = None,
    ) -> None:
        """Append a history entry."""
        ...

    def list(self, *, limit: int | None = None) -> list[dict[str, Any]]:
        """Return stored history entries."""
        ...

    def clear(self) -> None:
        """Clear all history entries."""
        ...


__all__ = ["HistoryProtocol"]
