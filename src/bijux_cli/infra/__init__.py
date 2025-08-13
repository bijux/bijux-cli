# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the public API for the Bijux CLI's infrastructure layer.

This module acts as the public facade for the concrete implementations of
various infrastructure services defined within the `bijux_cli.infra` package.
It aggregates these components into a single, stable namespace for convenient
importing and use by the application's service layer.

The exposed components provide implementations for core cross-cutting
concerns, including:
* **Observability:** `Observability` for structured logging.
* **Serialization:** `OrjsonSerializer` and `PyYAMLSerializer`.
* **Output:** `Emitter` for printing structured output.
* **Telemetry:** `LoggingTelemetry` and `NullTelemetry` for analytics.
* **Concurrency:** `ProcessPool` for managing worker processes.
* **Resilience:** `ExponentialBackoffRetryPolicy` for handling transient errors.
"""

from __future__ import annotations

from bijux_cli.infra.emitter import Emitter
from bijux_cli.infra.observability import Observability
from bijux_cli.infra.process import ProcessPool, get_process_pool
from bijux_cli.infra.retry import ExponentialBackoffRetryPolicy, TimeoutRetryPolicy
from bijux_cli.infra.serializer import OrjsonSerializer, PyYAMLSerializer, Redacted
from bijux_cli.infra.telemetry import LoggingTelemetry, NullTelemetry, TelemetryEvent

__all__ = [
    "Emitter",
    "Observability",
    "ProcessPool",
    "get_process_pool",
    "ExponentialBackoffRetryPolicy",
    "TimeoutRetryPolicy",
    "OrjsonSerializer",
    "PyYAMLSerializer",
    "Redacted",
    "LoggingTelemetry",
    "NullTelemetry",
    "TelemetryEvent",
]
