# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the application configuration service.

This module specifies the `ConfigProtocol`, a formal interface that any
configuration management service within the application must implement. This
ensures a consistent API for loading, accessing, modifying, and persisting
key-value settings, promoting modularity and testability.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class ConfigProtocol(Protocol):
    """Defines the contract for application configuration management.

    This interface specifies the methods for loading, accessing, modifying, and
    persisting configuration data from various sources (e.g., .env, JSON,
    or YAML files).
    """

    def load(self, path: str | Path | None = None) -> None:
        """Loads configuration from a specified file path.

        Args:
            path (str | Path | None): The path to the configuration file. If
                None, the service may load from a default location.

        Returns:
            None:
        """
        ...

    def get(self, key: str, default: Any = None) -> Any:
        """Retrieves a configuration value by its key.

        Args:
            key (str): The key of the value to retrieve.
            default (Any): The value to return if the key is not found.

        Returns:
            Any: The configuration value or the provided default.
        """
        ...

    def set(self, key: str, value: Any) -> None:
        """Sets a configuration key and persists the change.

        Args:
            key (str): The key to set or update.
            value (Any): The value to associate with the key.

        Returns:
            None:
        """
        ...

    def export(self, path: str | Path, out_format: str | None = None) -> None:
        """Exports the current configuration to a file.

        Args:
            path (str | Path): The path to the destination file.
            out_format (str | None): The desired output format (e.g., 'env').
                If None, the format may be inferred from the path.

        Returns:
            None:
        """
        ...

    def reload(self) -> None:
        """Reloads the configuration from its last-known source file."""
        ...

    def save(self) -> None:
        """Saves the current in-memory configuration state to its source file."""
        ...

    def delete(self, key: str) -> None:
        """Deletes a configuration key and persists the change.

        Args:
            key (str): The key to delete.

        Returns:
            None:
        """
        ...

    def unset(self, key: str) -> None:
        """Removes a configuration key from the current in-memory session.

        Unlike `delete`, this operation may not be immediately persisted to the
        source file.

        Args:
            key (str): The key to remove from the in-memory configuration.

        Returns:
            None:
        """
        ...

    def clear(self) -> None:
        """Clears all configuration data from memory."""
        ...

    def list_keys(self) -> list[str]:
        """Returns a list of all configuration keys.

        Returns:
            list[str]: A list of all keys present in the configuration.
        """
        ...

    def all(self) -> dict[str, str]:
        """Returns all configuration key-value pairs.

        Returns:
            dict[str, str]: A dictionary of all configuration data.
        """
        ...
