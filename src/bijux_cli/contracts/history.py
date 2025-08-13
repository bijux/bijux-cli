# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the command history management service.

This module specifies the `HistoryProtocol`, a formal interface that any
service responsible for recording, retrieving, and managing CLI command
history events must implement.
"""

from __future__ import annotations

from collections.abc import Sequence
from pathlib import Path
from typing import Any, Protocol, TypeVar, runtime_checkable

T = TypeVar("T")


@runtime_checkable
class HistoryProtocol(Protocol):
    """Defines the contract for command history management.

    This interface specifies the methods for recording, retrieving, and managing
    command history events, including persistence and import/export
    functionality.
    """

    def add(
        self,
        command: str,
        *,
        params: Sequence[str] | None = None,
        success: bool | None = True,
        return_code: int | None = 0,
        duration_ms: float | None = None,
    ) -> None:
        """Records a command execution event.

        Args:
            command (str): The full command string (e.g., "status --verbose").
            params (Sequence[str] | None): The raw argument vector as executed.
            success (bool | None): True if the command was successful.
            return_code (int | None): The integer exit code of the command.
            duration_ms (float | None): Total execution time in milliseconds.

        Returns:
            None:
        """
        ...

    def list(
        self,
        *,
        limit: int | None = 20,
        group_by: str | None = None,
        filter_cmd: str | None = None,
        sort: str | None = None,
    ) -> list[dict[str, Any]]:
        """Retrieves command history events.

        Args:
            limit (int | None): The maximum number of events to return.
            group_by (str | None): A field name to group events by.
            filter_cmd (str | None): A substring to filter commands by.
            sort (str | None): A field name to sort the results by.

        Returns:
            list[dict[str, Any]]: A list of history event dictionaries.
        """
        ...

    def clear(self) -> None:
        """Erases all command history events from the store."""
        ...

    def flush(self) -> None:
        """Persists any in-memory history data to permanent storage."""
        ...

    def export(self, path: Path) -> None:
        """Exports all history entries to a file.

        Args:
            path (Path): The target file path, which will be overwritten.

        Returns:
            None:
        """
        ...

    def import_(self, path: Path) -> None:
        """Imports history entries from a file, replacing existing entries.

        Args:
            path (Path): The source file path containing history data.

        Returns:
            None:
        """
        ...
