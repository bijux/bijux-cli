# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Telemetry configuration helpers."""

from __future__ import annotations

import os

from bijux_cli.core.di import DIContainer
from bijux_cli.infra.telemetry import LoggingTelemetry, NoopTelemetry
from bijux_cli.services.contracts import TelemetryProtocol


def resolve_telemetry(
    di: DIContainer, enabled: bool | None = None
) -> TelemetryProtocol:
    """Resolve telemetry based on an opt-in flag or environment variable."""
    if enabled is None:
        enabled = os.getenv("BIJUXCLI_TELEMETRY", "").lower() in {"1", "true", "yes"}
    if enabled:
        return di.resolve(LoggingTelemetry)
    return di.resolve(NoopTelemetry)


__all__ = ["resolve_telemetry"]
