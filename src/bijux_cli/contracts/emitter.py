# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the structured output emission service.

This module specifies the `EmitterProtocol`, a formal interface that any
service responsible for serializing data payloads (e.g., to JSON or YAML)
and emitting them to an output stream must implement.
"""

from __future__ import annotations

from typing import Any, Protocol, TypeVar, runtime_checkable

from bijux_cli.core.enums import OutputFormat

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
        fmt: OutputFormat | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "Emitting output",
        output: str | None = None,
        **context: Any,
    ) -> None:
        """Serializes and emits a structured data payload.

        Args:
            payload (Any): The data payload to serialize and emit.
            fmt (OutputFormat | None): The output format for serialization.
            pretty (bool): If True, pretty-prints the output with indentation.
            level (str): The log level for any accompanying message.
            message (str): A descriptive message for logging.
            output (str | None): An optional pre-formatted string to emit
                instead of serializing the payload.
            **context (Any): Additional key-value pairs for structured logging.

        Returns:
            None:
        """
        ...

    def flush(self) -> None:
        """Flushes any buffered output to its destination stream."""
        ...
