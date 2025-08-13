# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the public API for all service and infrastructure contracts.

This module acts as the public facade for the application's service contracts,
which are defined using Python's `Protocol`. It aggregates all individual
protocol definitions from the `bijux_cli.contracts` submodules into a single,
stable namespace.

By importing from this module, other parts of the application can depend on
these abstract interfaces without being coupled to the concrete implementation
details or the internal structure of the contracts package.
"""

from __future__ import annotations

from bijux_cli.contracts.audit import AuditProtocol
from bijux_cli.contracts.config import ConfigProtocol
from bijux_cli.contracts.context import ContextProtocol
from bijux_cli.contracts.docs import DocsProtocol
from bijux_cli.contracts.doctor import DoctorProtocol
from bijux_cli.contracts.emitter import EmitterProtocol
from bijux_cli.contracts.history import HistoryProtocol
from bijux_cli.contracts.memory import MemoryProtocol
from bijux_cli.contracts.observability import ObservabilityProtocol
from bijux_cli.contracts.process import ProcessPoolProtocol
from bijux_cli.contracts.registry import RegistryProtocol
from bijux_cli.contracts.retry import RetryPolicyProtocol
from bijux_cli.contracts.serializer import SerializerProtocol
from bijux_cli.contracts.telemetry import TelemetryProtocol

__all__ = [
    "AuditProtocol",
    "ConfigProtocol",
    "ContextProtocol",
    "DocsProtocol",
    "DoctorProtocol",
    "EmitterProtocol",
    "HistoryProtocol",
    "MemoryProtocol",
    "ObservabilityProtocol",
    "ProcessPoolProtocol",
    "RegistryProtocol",
    "RetryPolicyProtocol",
    "SerializerProtocol",
    "TelemetryProtocol",
]
