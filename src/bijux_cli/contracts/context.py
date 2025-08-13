# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the request-scoped context service.

This module specifies the `ContextProtocol`, a formal interface for services
that manage contextual data associated with a specific operation or request.
This pattern allows for state to be carried implicitly through an application's
call stack.
"""

from __future__ import annotations

from typing import Any, Protocol, Self, runtime_checkable


@runtime_checkable
class ContextProtocol(Protocol):
    """Defines the contract for a request-scoped context object.

    This interface specifies methods for a context that can store arbitrary
    key-value data and supports both synchronous (`with`) and asynchronous
    (`async with`) context management patterns.
    """

    def set(self, key: str, value: Any) -> None:
        """Sets a key-value pair in the context.

        Args:
            key (str): The key to set.
            value (Any): The value to associate with the key.

        Returns:
            None:
        """
        ...

    def get(self, key: str) -> Any:
        """Retrieves a value by key from the context.

        Args:
            key (str): The key of the value to retrieve.

        Returns:
            Any: The value associated with the key.
        """
        ...

    def clear(self) -> None:
        """Clears all data from the context."""
        ...

    def __enter__(self) -> Self:
        """Enters the synchronous context manager.

        Returns:
            Self: The context instance itself.
        """
        ...

    def __exit__(self, _exc_type: Any, _exc_value: Any, traceback: Any) -> None:
        """Exits the synchronous context manager.

        Args:
            _exc_type (Any): The exception type, if any.
            _exc_value (Any): The exception value, if any.
            traceback (Any): The traceback, if any.

        Returns:
            None:
        """
        ...

    async def __aenter__(self) -> Self:
        """Enters the asynchronous context manager.

        Returns:
            Self: The context instance itself.
        """
        ...

    async def __aexit__(self, _exc_type: Any, _exc_value: Any, traceback: Any) -> None:
        """Exits the asynchronous context manager.

        Args:
            _exc_type (Any): The exception type, if any.
            _exc_value (Any): The exception value, if any.
            traceback (Any): The traceback, if any.

        Returns:
            None:
        """
        ...
