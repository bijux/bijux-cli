# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Service-level protocol contracts."""

from __future__ import annotations

from collections.abc import Coroutine
from typing import Any, Protocol, Self, runtime_checkable

from structlog.typing import FilteringBoundLogger

from bijux_cli.core.enums import LogLevel


@runtime_checkable
class TelemetryProtocol(Protocol):
    """Fire-and-forget telemetry sink."""

    def event(
        self, name: str, payload: dict[str, Any]
    ) -> None | Coroutine[Any, Any, None]:
        """Record a telemetry event."""
        ...

    def flush(self) -> None:
        """Flush buffered events."""
        ...

    def enable(self) -> None:
        """Enable telemetry output."""
        ...


@runtime_checkable
class ObservabilityProtocol(Protocol):
    """Structured logging facade."""

    @classmethod
    def setup(cls, *, log_level: LogLevel, telemetry: TelemetryProtocol) -> Self:
        """Configure and return a logging instance."""
        ...

    def get_logger(self) -> FilteringBoundLogger | None:
        """Return the bound logger instance."""
        ...

    def bind(self, **_kv: Any) -> Self:
        """Bind context keys for future log entries."""
        ...

    def log(self, level: str, msg: str, *, extra: dict[str, Any] | None) -> Self:
        """Emit a structured log entry."""
        ...

    def close(self) -> None:
        """Close the logger and release resources."""
        ...

    def set_telemetry(self, telemetry: TelemetryProtocol) -> Self:
        """Attach a telemetry sink."""
        ...


__all__ = [
    "ObservabilityProtocol",
    "TelemetryProtocol",
]
