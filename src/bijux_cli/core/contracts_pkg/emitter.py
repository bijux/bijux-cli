# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the structured output emission service.

This module specifies the `EmitterProtocol`, a formal interface that any
service responsible for serializing data payloads (e.g., to JSON or YAML)
and emitting them to an output stream must implement.
"""

from __future__ import annotations

from typing import Any, Protocol, TypeVar, runtime_checkable

T = TypeVar("T")


@runtime_checkable
class EmitterProtocol(Protocol):
    """Defines the contract for emitting structured output.

    This interface specifies the methods for serializing and emitting data in
    various formats, often integrating with a logging or telemetry system.
    """

    def emit(
        self,
        payload: Any,
        *,
        fmt: str | None,
        pretty: bool,
        level: str,
        message: str,
        output: str | None,
        **context: Any,
    ) -> None:
        """Serialize and emit a structured payload."""
        ...

    def flush(self) -> None:
        """Flush any buffered output."""
        ...
