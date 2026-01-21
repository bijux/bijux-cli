# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the history service."""

from __future__ import annotations

from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class HistoryProtocol(Protocol):
    """Defines the contract for the history service."""

    def add(
        self,
        *,
        command: str,
        params: list[str],
        success: bool,
        return_code: int,
        duration_ms: float,
        raw: dict[str, Any] | None = None,
    ) -> None: ...

    def list(self, *, limit: int | None = None) -> list[dict[str, Any]]: ...

    def clear(self) -> None: ...


__all__ = ["HistoryProtocol"]
