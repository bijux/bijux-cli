# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a thread-safe, file-persisted key-value store.

This module defines the `Memory` class, a concrete implementation of the
`MemoryProtocol`. It uses a dictionary for in-memory storage, protected by a
`threading.Lock` for thread safety. Unlike a purely transient store, this
implementation persists the entire key-value store to a JSON file on every
write operation, allowing state to survive across different CLI invocations.
"""

from __future__ import annotations

import json
from threading import Lock
from typing import Any

from injector import inject

from bijux_cli.contracts import MemoryProtocol
from bijux_cli.core.paths import MEMORY_FILE


class Memory(MemoryProtocol):
    """Implements `MemoryProtocol` with a thread-safe, file-backed dictionary.

    This service provides a simple key-value store that is both thread-safe
    and persistent to a JSON file (`~/.bijux/.memory.json`).

    Attributes:
        _store (dict[str, Any]): The in-memory dictionary holding the data.
        _lock (Lock): A lock to ensure thread-safe access to the store.
    """

    @inject
    def __init__(self) -> None:
        """Initializes the service, loading existing data from the persistence file."""
        MEMORY_FILE.parent.mkdir(parents=True, exist_ok=True)
        try:
            with MEMORY_FILE.open("r") as f:
                self._store: dict[str, Any] = json.load(f)
        except (FileNotFoundError, json.JSONDecodeError):
            self._store = {}
        self._lock = Lock()

    def get(self, key: str) -> Any:
        """Retrieves a value by its key in a thread-safe manner.

        Args:
            key (str): The key of the value to retrieve.

        Returns:
            Any: The value associated with the key.

        Raises:
            KeyError: If the key does not exist in the store.
        """
        with self._lock:
            if key not in self._store:
                raise KeyError(f"Memory key not found: {key}")
            return self._store[key]

    def set(self, key: str, value: Any) -> None:
        """Sets a key-value pair and persists the change to disk.

        If the key already exists, its value is overwritten. This operation
        is thread-safe.

        Args:
            key (str): The key for the value being set.
            value (Any): The value to store.

        Returns:
            None:
        """
        with self._lock:
            self._store[key] = value
            self._persist()

    def delete(self, key: str) -> None:
        """Deletes a key-value pair and persists the change to disk.

        This operation is thread-safe.

        Args:
            key (str): The key of the value to delete.

        Raises:
            KeyError: If the key does not exist in the store.
        """
        with self._lock:
            if key not in self._store:
                raise KeyError(f"Memory key not found: {key}")
            del self._store[key]
            self._persist()

    def clear(self) -> None:
        """Removes all key-value pairs and persists the change to disk.

        This operation is thread-safe.
        """
        with self._lock:
            self._store.clear()
            self._persist()

    def keys(self) -> list[str]:
        """Returns a list of all keys currently in the store.

        This operation is thread-safe.

        Returns:
            list[str]: A list of all string keys.
        """
        with self._lock:
            return list(self._store.keys())

    def _persist(self) -> None:
        """Writes the current in-memory store to the JSON persistence file.

        Note:
            This method is not thread-safe on its own and should only be
            called from within a block that holds `self._lock`.
        """
        with MEMORY_FILE.open("w") as f:
            json.dump(self._store, f)


__all__ = ["Memory"]
