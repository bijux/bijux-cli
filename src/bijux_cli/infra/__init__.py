# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the public API for the Bijux CLI's infrastructure layer.

The infra package is intentionally minimal: only OS/IO utilities live here.
Service implementations that depend on core protocols or errors are housed
under `services/` instead of `infra/`.
"""

from __future__ import annotations

from bijux_cli.infra.emitter import ConsoleEmitter, NullEmitter
from bijux_cli.infra.fs import NoopFileSystem
from bijux_cli.infra.process import NoopProcessExecutor, ProcessPool, get_process_pool
from bijux_cli.infra.retry import (
    ExponentialBackoffRetryPolicy,
    NoopRetryPolicy,
    TimeoutRetryPolicy,
)
from bijux_cli.infra.serializer import (
    NoopSerializer,
    OrjsonSerializer,
    PyYAMLSerializer,
    Redacted,
    serializer_for,
)
from bijux_cli.infra.telemetry import LoggingTelemetry, NoopTelemetry, TelemetryEvent
from bijux_cli.infra.terminal import NoopTerminal

__all__ = [
    "ConsoleEmitter",
    "NullEmitter",
    "NoopFileSystem",
    "NoopProcessExecutor",
    "ProcessPool",
    "get_process_pool",
    "NoopRetryPolicy",
    "TimeoutRetryPolicy",
    "ExponentialBackoffRetryPolicy",
    "OrjsonSerializer",
    "PyYAMLSerializer",
    "Redacted",
    "NoopSerializer",
    "serializer_for",
    "NoopTelemetry",
    "LoggingTelemetry",
    "TelemetryEvent",
    "NoopTerminal",
]
