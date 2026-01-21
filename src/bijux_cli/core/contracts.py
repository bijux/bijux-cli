# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Protocol and contract definitions (facade)."""

from __future__ import annotations

from bijux_cli.core.contracts_pkg.audit import AuditProtocol
from bijux_cli.core.contracts_pkg.config import ConfigProtocol
from bijux_cli.core.contracts_pkg.context import ContextProtocol
from bijux_cli.core.contracts_pkg.docs import DocsProtocol
from bijux_cli.core.contracts_pkg.doctor import DoctorProtocol
from bijux_cli.core.contracts_pkg.emitter import EmitterProtocol
from bijux_cli.core.contracts_pkg.history import HistoryProtocol
from bijux_cli.core.contracts_pkg.memory import MemoryProtocol
from bijux_cli.core.contracts_pkg.observability import ObservabilityProtocol
from bijux_cli.core.contracts_pkg.process import ProcessPoolProtocol
from bijux_cli.core.contracts_pkg.registry import RegistryProtocol
from bijux_cli.core.contracts_pkg.retry import RetryPolicyProtocol
from bijux_cli.core.contracts_pkg.serializer import SerializerProtocol
from bijux_cli.core.contracts_pkg.telemetry import TelemetryProtocol

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
