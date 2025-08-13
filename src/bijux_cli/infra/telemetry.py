# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides concrete telemetry service implementations for event tracking.

This module defines concrete classes that implement the `TelemetryProtocol`.
It offers different strategies for handling telemetry events, allowing the
application's analytics behavior to be configured easily.

Key components include:
    * `TelemetryEvent`: An enumeration of all standardized event names, providing
        a single source of truth for telemetry event types.
    * `NullTelemetry`: A no-op implementation that silently discards all events,
        useful for disabling telemetry entirely.
    * `LoggingTelemetry`: An implementation that forwards all telemetry events to
        the application's structured logging service.
"""

from __future__ import annotations

from enum import Enum
from typing import Any

from injector import inject

from bijux_cli.contracts import ObservabilityProtocol, TelemetryProtocol


class TelemetryEvent(str, Enum):
    """Defines standardized telemetry event names for tracking CLI activities."""

    CLI_STARTED = "cli_started"
    CLI_ERROR = "cli_error"
    CLI_INTERRUPTED = "cli_interrupted"
    CLI_SYSTEM_EXIT = "cli_system_exit"
    CLI_UNEXPECTED_ERROR = "cli_unexpected_error"
    CLI_SHUTDOWN_FAILED = "cli_shutdown_failed"
    ENGINE_INITIALIZED = "engine_initialized"
    ENGINE_SHUTDOWN = "engine_shutdown"
    PLUGINS_LIST_COMMAND = "cmd/plugins/list"
    PLUGINS_LIST_COMMAND_FAILED = "cmd/err/plugins/list"
    PLUGINS_INFO_COMMAND = "cmd/plugins/info"
    PLUGINS_INFO_COMMAND_FAILED = "cmd/err/plugins/info"
    PLUGINS_INFO_NOT_FOUND = "cmd/err/plugins/info/not_found"
    PLUGINS_INSTALL_COMMAND = "cmd/plugins/install"
    PLUGINS_INSTALL_COMMAND_FAILED = "cmd/err/plugins/install"
    PLUGINS_UNINSTALL_COMMAND = "cmd/plugins/uninstall"
    PLUGINS_UNINSTALL_COMMAND_FAILED = "cmd/err/plugins/uninstall"
    PLUGINS_UNINSTALL_NOT_FOUND = "cmd/err/plugins/uninstall/not_found"
    PLUGINS_CHECK_COMMAND = "cmd/plugins/check"
    PLUGINS_CHECK_COMMAND_FAILED = "cmd/err/plugins/check"
    PLUGINS_CHECK_NOT_FOUND = "cmd/err/plugins/check/not_found"
    PLUGINS_SCAFFOLD_COMMAND = "cmd/plugins/scaffold"
    PLUGINS_SCAFFOLD_COMMAND_FAILED = "cmd/err/plugins/scaffold"
    PLUGINS_SCAFFOLD_DIR_EXISTS = "cmd/err/plugins/scaffold/dir_exists"
    CONFIG_COMMAND = "cmd/config"
    CONFIG_COMMAND_FAILED = "cmd/err/config"
    AUDIT_COMMAND = "cmd/audit"
    AUDIT_COMMAND_FAILED = "cmd/err/audit"
    DOCTOR_COMMAND = "cmd/doctor"
    DOCTOR_COMMAND_FAILED = "cmd/err/doctor"
    VERSION_COMMAND = "cmd/version"
    VERSION_COMMAND_FAILED = "cmd/err/version"
    STATUS_COMMAND = "cmd/status"
    STATUS_COMMAND_FAILED = "cmd/err/status"
    SLEEP_COMMAND = "cmd/test/sleep"
    SLEEP_COMMAND_FAILED = "cmd/err/test/sleep"
    HISTORY_COMMAND = "cmd/history"
    HISTORY_COMMAND_FAILED = "cmd/err/history"
    REPL_COMMAND = "cmd/repl"
    REPL_EXIT = "cmd/repl/exit"
    REPL_COMMAND_NOT_FOUND = "cmd/err/repl/not_found"
    DEV_COMMAND_EXECUTED = "cmd/dev"
    DEV_COMMAND_FAILED = "cmd/err/dev"
    MEMORY_COMMAND_EXECUTED = "cmd/memory"
    MEMORY_COMMAND_FAILED = "cmd/err/memory"
    HELP_COMMAND = "cmd/help"
    HELP_COMMAND_FAILED = "cmd/err/help"
    PLUGIN_STARTED = "plugin_started"
    PLUGIN_SHUTDOWN = "plugin_shutdown"
    PLUGIN_INSTALLED = "plugin_installed"
    PLUGIN_LOADED = "plugin_loaded"
    PLUGIN_CLI_REGISTERED = "plugin_cli_registered"
    PLUGIN_LOAD_FAILED = "plugin_load_failed"


class NullTelemetry(TelemetryProtocol):  # pyright: ignore[reportUntypedBaseClass]
    """A no-op telemetry service that discards all events.

    This implementation of `TelemetryProtocol` can be used to effectively
    disable analytics and event tracking.
    """

    def event(self, name: str | TelemetryEvent, payload: dict[str, Any]) -> None:
        """Discards the telemetry event.

        Args:
            name (str | TelemetryEvent): The event name (ignored).
            payload (dict[str, Any]): The event data (ignored).

        Returns:
            None:
        """
        return

    def flush(self) -> None:
        """Performs a no-op flush operation."""
        return

    def enable(self) -> None:
        """Performs a no-op enable operation."""
        return


class LoggingTelemetry(TelemetryProtocol):  # pyright: ignore[reportUntypedBaseClass]
    """A telemetry service that logs events via the `Observability` service.

    This implementation of `TelemetryProtocol` forwards all telemetry events
    to the structured logger as debug-level messages.

    Attributes:
        _obs (ObservabilityProtocol): The logging service instance.
        _buffer (list): A buffer to store events (currently only cleared on flush).
    """

    @inject
    def __init__(self, observability: ObservabilityProtocol):
        """Initializes the `LoggingTelemetry` service.

        Args:
            observability (ObservabilityProtocol): The service for logging events.
        """
        self._obs = observability
        self._buffer: list[tuple[str, dict[str, Any]]] = []

    def event(self, name: str | TelemetryEvent, payload: dict[str, Any]) -> None:
        """Logs a telemetry event at the 'debug' level.

        Args:
            name (str | TelemetryEvent): The event name or enum member.
            payload (dict[str, Any]): The event data dictionary.

        Returns:
            None:
        """
        event_name = name.value if isinstance(name, TelemetryEvent) else name
        self._obs.log("debug", f"Telemetry event: {event_name}", extra=payload)
        self._buffer.append((event_name, payload))

    def flush(self) -> None:
        """Clears the internal buffer of telemetry events."""
        self._buffer.clear()

    def enable(self) -> None:
        """Performs a no-op enable operation."""
        return


__all__ = ["TelemetryEvent", "NullTelemetry", "LoggingTelemetry"]
