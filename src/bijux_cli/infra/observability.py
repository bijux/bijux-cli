# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the concrete implementation of the observability and logging service.

This module defines the `Observability` class, which implements the
`ObservabilityProtocol`. It serves as the primary interface for structured
logging throughout the application, using `structlog` as its underlying engine.
It can also be configured to forward log entries to a telemetry backend,
unifying logging and event tracking.
"""

from __future__ import annotations

from typing import Any, Self

from injector import inject
import structlog
from structlog.typing import FilteringBoundLogger

from bijux_cli.contracts import ObservabilityProtocol, TelemetryProtocol
from bijux_cli.core.exceptions import ServiceError
from bijux_cli.infra.telemetry import NullTelemetry


class Observability(ObservabilityProtocol):  # pyright: ignore[reportUntypedBaseClass]
    """A structured logging service integrating `structlog` and telemetry.

    This class wraps a `structlog` logger to produce structured log entries.
    If configured with a telemetry backend, it also forwards these events for
    analytics and monitoring.

    Attributes:
        _logger (FilteringBoundLogger): The underlying `structlog` logger instance.
        _telemetry (TelemetryProtocol): The telemetry service for event forwarding.
            Defaults to a `NullTelemetry` instance that does nothing.
    """

    @inject
    def __init__(self, *, debug: bool = False) -> None:
        """Initializes the observability service.

        Args:
            debug (bool): If True, configures the service for debug-level
                logging.
        """
        self._logger: FilteringBoundLogger = structlog.get_logger("bijux_cli")
        self._telemetry: TelemetryProtocol = NullTelemetry()

    def set_telemetry(self, telemetry: TelemetryProtocol) -> Self:
        """Attaches a telemetry backend for forwarding log events.

        This allows the service to be "upgraded" from a simple logger to a full
        observability tool after its initial creation.

        Args:
            telemetry (TelemetryProtocol): The telemetry service to receive events.

        Returns:
            Self: The service instance, allowing for method chaining.
        """
        self._telemetry = telemetry
        return self

    @classmethod
    def setup(cls, *, debug: bool = False) -> Self:
        """Instantiates and configures an `Observability` service.

        Args:
            debug (bool): If True, enables debug-level logging.

        Returns:
            Self: A new, configured `Observability` instance.
        """
        return cls(debug=debug)

    def get_logger(self) -> FilteringBoundLogger:
        """Retrieves the underlying `structlog` logger instance.

        Returns:
            FilteringBoundLogger: The `structlog` logger, which can be used
                directly if needed.
        """
        return self._logger

    def bind(self, **kwargs: Any) -> Self:
        """Binds context key-value pairs to all subsequent log entries.

        Args:
            **kwargs (Any): Context values to include in each log entry.

        Returns:
            Self: The service instance, allowing for method chaining.
        """
        self._logger = self._logger.bind(**kwargs)
        return self

    def log(
        self,
        level: str,
        msg: str,
        *,
        extra: dict[str, Any] | None = None,
    ) -> Self:
        """Logs a structured message and emits a corresponding telemetry event.

        Args:
            level (str): The severity level of the log (e.g., 'debug', 'info',
                'warning', 'error', 'critical').
            msg (str): The log message.
            extra (dict[str, Any] | None): Additional context to include in the
                log entry.

        Returns:
            Self: The service instance, allowing for method chaining.

        Raises:
            ServiceError: If `level` is not a valid log level name.
        """
        log_func = getattr(self._logger, level.lower(), None)
        if not callable(log_func):
            raise ServiceError(f"Invalid log level: {level}")

        log_context = extra or {}
        if log_context:
            log_func(msg, **log_context)
        else:
            log_func(msg)

        if not isinstance(self._telemetry, NullTelemetry):
            telemetry_payload = {"level": level, "message": msg}
            telemetry_payload.update(log_context)
            self._telemetry.event("LOG_EMITTED", telemetry_payload)

        return self

    def close(self) -> None:
        """Logs the shutdown of the observability service.

        Note:
            In this implementation, this method only logs a debug message and
            does not perform resource cleanup like flushing. Flushing is
            handled by the telemetry service's own lifecycle methods.
        """
        self._logger.debug("Observability shutdown")


__all__ = ["Observability"]
