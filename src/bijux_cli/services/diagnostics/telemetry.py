# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Telemetry configuration helpers."""

from __future__ import annotations

import os

from bijux_cli.core.contracts import TelemetryProtocol
from bijux_cli.app.di import DIContainer
from bijux_cli.infra.telemetry import LoggingTelemetry, NoopTelemetry


def resolve_telemetry(di: DIContainer, enabled: bool | None = None) -> TelemetryProtocol:
    """Resolve telemetry based on an opt-in flag or environment variable."""
    if enabled is None:
        enabled = os.getenv("BIJUXCLI_TELEMETRY", "").lower() in {"1", "true", "yes"}
    if enabled:
        return di.resolve(LoggingTelemetry)
    return di.resolve(NoopTelemetry)


__all__ = ["resolve_telemetry"]
