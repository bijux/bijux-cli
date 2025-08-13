# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the fire-and-forget telemetry service.

This module specifies the `TelemetryProtocol`, a formal interface that any
service responsible for collecting and managing "fire-and-forget" telemetry
or analytics events must implement.
"""

from __future__ import annotations

from collections.abc import Coroutine
from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class TelemetryProtocol(Protocol):
    """Defines the contract for fire-and-forget analytics collection.

    This interface specifies the methods for recording telemetry events and
    managing the lifecycle of the telemetry service.
    """

    def event(
        self, name: str, payload: dict[str, Any]
    ) -> None | Coroutine[Any, Any, None]:
        """Records a telemetry event.

        Args:
            name (str): The name of the event (e.g., "COMMAND_START").
            payload (dict[str, Any]): A dictionary of key-value pairs
                containing the event data.

        Returns:
            Either None (sync) or an awaitable resolving to None (async).
        """
        ...

    def flush(self) -> None:
        """Flushes any buffered telemetry events to their destination."""
        ...

    def enable(self) -> None:
        """Enables the collection of telemetry data."""
        ...
