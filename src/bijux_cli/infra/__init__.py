# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Provides the public API for the Bijux CLI's infrastructure layer.

The infra package is intentionally minimal: only OS/IO utilities live here.
Service implementations that depend on core protocols or errors are housed
under `services/` instead of `infra/`.
"""

from __future__ import annotations

from bijux_cli.infra.emitter import ConsoleEmitter, NullEmitter
from bijux_cli.infra.process import ProcessPool
from bijux_cli.infra.retry import (
    ExponentialBackoffRetryPolicy,
    NoopRetryPolicy,
    TimeoutRetryPolicy,
)
from bijux_cli.infra.serializer import (
    OrjsonSerializer,
    PyYAMLSerializer,
    serializer_for,
)
from bijux_cli.infra.telemetry import LoggingTelemetry, NoopTelemetry, TelemetryEvent

__all__ = [
    "ConsoleEmitter",
    "NullEmitter",
    "ProcessPool",
    "NoopRetryPolicy",
    "TimeoutRetryPolicy",
    "ExponentialBackoffRetryPolicy",
    "OrjsonSerializer",
    "PyYAMLSerializer",
    "serializer_for",
    "NoopTelemetry",
    "LoggingTelemetry",
    "TelemetryEvent",
]
