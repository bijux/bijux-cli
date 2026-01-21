# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the public API for the Bijux CLI's infrastructure layer.

The infra package is intentionally minimal: only OS/IO utilities live here.
Service implementations that depend on core protocols or errors are housed
under `services/` instead of `infra/`.
"""

from __future__ import annotations

from bijux_cli.infra.emitter import ConsoleEmitter, Emitter, NullEmitter
from bijux_cli.infra.fs import FileSystem, NoopFileSystem
from bijux_cli.infra.process import (
    NoopProcessExecutor,
    ProcessExecutor,
    ProcessPool,
    get_process_pool,
)
from bijux_cli.infra.retry import ExponentialBackoffRetryPolicy, NoopRetryPolicy, RetryPolicy, TimeoutRetryPolicy
from bijux_cli.infra.serializer import (
    NoopSerializer,
    OrjsonSerializer,
    PyYAMLSerializer,
    Redacted,
    Serializer,
    serializer_for,
)
from bijux_cli.infra.telemetry import LoggingTelemetry, NoopTelemetry, Telemetry, TelemetryEvent
from bijux_cli.infra.terminal import NoopTerminal, Terminal

__all__ = [
    "Emitter",
    "ConsoleEmitter",
    "NullEmitter",
    "FileSystem",
    "NoopFileSystem",
    "ProcessExecutor",
    "NoopProcessExecutor",
    "ProcessPool",
    "get_process_pool",
    "RetryPolicy",
    "NoopRetryPolicy",
    "TimeoutRetryPolicy",
    "ExponentialBackoffRetryPolicy",
    "OrjsonSerializer",
    "PyYAMLSerializer",
    "Redacted",
    "Serializer",
    "NoopSerializer",
    "serializer_for",
    "Telemetry",
    "NoopTelemetry",
    "LoggingTelemetry",
    "TelemetryEvent",
    "Terminal",
    "NoopTerminal",
]
