# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the transient in-memory key-value service.

This module specifies the `MemoryProtocol`, a formal interface that any
service providing a non-persistent, in-memory key-value store must
implement. This is used for managing ephemeral state within the CLI.
"""

from __future__ import annotations

from typing import Any, Protocol, TypeVar, runtime_checkable

T = TypeVar("T")


@runtime_checkable
class MemoryProtocol(Protocol):
    """Defines the contract for a simple in-memory key-value store.

    This interface specifies the essential methods for a basic key-value data
    storage service. It is used for managing ephemeral state within the CLI
    that does not need to persist across sessions.
    """

    def get(self, key: str) -> Any:
        """Retrieves a value by its key.

        Args:
            key (str): The key of the value to retrieve.

        Returns:
            Any: The value associated with the key.

        Raises:
            KeyError: If the key does not exist in the store.
        """
        ...

    def set(self, key: str, value: Any) -> None:
        """Sets a key-value pair in the store.

        If the key already exists, its value will be overwritten.

        Args:
            key (str): The key for the value being set.
            value (Any): The value to store.

        Returns:
            None:
        """
        ...

    def delete(self, key: str) -> None:
        """Deletes a key-value pair by its key.

        Args:
            key (str): The key of the value to delete.

        Raises:
            KeyError: If the key does not exist in the store.
        """
        ...

    def clear(self) -> None:
        """Removes all key-value pairs from the store."""
        ...

    def keys(self) -> list[str]:
        """Returns a list of all keys currently in the store.

        Returns:
            list[str]: A list of all string keys.
        """
        ...
