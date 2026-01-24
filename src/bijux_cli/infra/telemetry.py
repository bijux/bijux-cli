# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Telemetry adapter interfaces and default implementations."""

from __future__ import annotations

from enum import Enum
from typing import Any


class TelemetryEvent(str, Enum):
    """Standardized telemetry event names."""

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


class NoopTelemetry:
    """No-op telemetry adapter."""

    def event(self, name: str | TelemetryEvent, payload: dict[str, Any]) -> None:
        """Record a telemetry event and do nothing."""
        return None

    def flush(self) -> None:
        """Flush buffered events (no-op)."""
        return None

    def enable(self) -> None:
        """Enable telemetry (no-op)."""
        return None


class LoggingTelemetry:
    """Telemetry adapter that logs events via an observability sink."""

    def __init__(self, observability: Any) -> None:
        """Initialize with an observability sink."""
        self._observability = observability
        self._buffer: list[tuple[str, dict[str, Any]]] = []

    def event(self, name: str | TelemetryEvent, payload: dict[str, Any]) -> None:
        """Record and log a telemetry event."""
        event = name.value if isinstance(name, TelemetryEvent) else str(name)
        self._buffer.append((event, payload))
        self._observability.log(
            "debug", f"telemetry:{event}", extra={"event": event, **payload}
        )

    def flush(self) -> None:
        """Clear buffered telemetry events."""
        self._buffer.clear()

    def enable(self) -> None:
        """Enable telemetry (no-op for logging adapter)."""
        return None


__all__ = [
    "TelemetryEvent",
    "NoopTelemetry",
    "LoggingTelemetry",
]
