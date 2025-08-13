# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the structured logging and observability service.

This module specifies the `ObservabilityProtocol`, a formal interface that any
service providing structured logging capabilities must implement. It ensures a
consistent API for configuring loggers, binding contextual data, and emitting
log messages throughout the application.
"""

from __future__ import annotations

from typing import Any, Protocol, Self, runtime_checkable

from structlog.typing import FilteringBoundLogger

from bijux_cli.contracts.telemetry import TelemetryProtocol


@runtime_checkable
class ObservabilityProtocol(Protocol):
    """Defines the contract for a structured logging facade.

    This interface specifies methods for configuring logging, binding contextual
    data, and emitting log messages in a structured format.
    """

    @classmethod
    def setup(cls, *, debug: bool = False) -> Self:
        """Creates and configures a new instance of the observability service.

        Args:
            debug (bool): If True, configures the logger for debug-level output.

        Returns:
            Self: A new, configured instance of the service.
        """
        ...

    def get_logger(self) -> FilteringBoundLogger | None:
        """Returns the underlying logger instance.

        Returns:
            FilteringBoundLogger | None: The logger instance (e.g., from
                `structlog`) or None if it has not been configured.
        """
        ...

    def bind(self, **_kv: Any) -> Self:
        """Binds context key-value pairs to the logger for future log entries.

        Args:
            **_kv (Any): Key-value pairs to bind to the logging context.

        Returns:
            Self: The service instance, allowing for method chaining.
        """
        ...

    def log(
        self,
        level: str,
        msg: str,
        *,
        extra: dict[str, Any] | None = None,
    ) -> Self:
        """Logs a message with the specified level and context.

        Args:
            level (str): The log level (e.g., 'debug', 'info', 'error').
            msg (str): The message to log.
            extra (dict[str, Any] | None): An optional dictionary of
                additional context to include in the log entry.

        Returns:
            Self: The service instance, allowing for method chaining.
        """
        ...

    def close(self) -> None:
        """Closes the logger and releases any associated resources."""
        ...

    def set_telemetry(self, telemetry: TelemetryProtocol) -> Self:
        """Sets the telemetry service instance for integration.

        Args:
            telemetry (TelemetryProtocol): An instance of the telemetry service.

        Returns:
            Self: The service instance, allowing for method chaining.
        """
        ...
