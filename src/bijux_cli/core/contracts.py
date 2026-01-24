# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Core protocol contracts for Bijux CLI."""

from __future__ import annotations

from typing import Any, Protocol, Self, runtime_checkable


@runtime_checkable
class ExecutionContext(Protocol):
    """Execution-scoped context carrier."""

    def set(self, key: str, value: Any) -> None:
        """Store a value in the context."""
        ...

    def get(self, key: str) -> Any:
        """Retrieve a value from the context."""
        ...

    def clear(self) -> None:
        """Clear all context data."""
        ...

    def __enter__(self) -> Self:
        """Enter the synchronous context manager."""
        ...

    def __exit__(self, _exc_type: Any, _exc_value: Any, traceback: Any) -> None:
        """Exit the synchronous context manager."""
        ...

    async def __aenter__(self) -> Self:
        """Enter the async context manager."""
        ...

    async def __aexit__(self, _exc_type: Any, _exc_value: Any, traceback: Any) -> None:
        """Exit the async context manager."""
        ...


__all__ = [
    "ExecutionContext",
]
