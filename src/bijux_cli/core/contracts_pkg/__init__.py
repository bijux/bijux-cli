# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the public API for core contracts.

This module aggregates the core-facing protocols (cross-cutting behavior and
infra-facing interfaces) into a single, stable namespace. Service-specific
contracts live under `services/<name>/contracts.py`.
"""

from __future__ import annotations

from bijux_cli.core.contracts_pkg.context import ContextProtocol
from bijux_cli.core.contracts_pkg.emitter import EmitterProtocol
from bijux_cli.core.contracts_pkg.fs import FileSystemProtocol
from bijux_cli.core.contracts_pkg.observability import ObservabilityProtocol
from bijux_cli.core.contracts_pkg.process import ProcessPoolProtocol
from bijux_cli.core.contracts_pkg.registry import RegistryProtocol
from bijux_cli.core.contracts_pkg.retry import RetryPolicyProtocol
from bijux_cli.core.contracts_pkg.serializer import SerializerProtocol
from bijux_cli.core.contracts_pkg.telemetry import TelemetryProtocol
from bijux_cli.core.contracts_pkg.terminal import TerminalProtocol

__all__ = [
    "ContextProtocol",
    "EmitterProtocol",
    "FileSystemProtocol",
    "ObservabilityProtocol",
    "ProcessPoolProtocol",
    "RegistryProtocol",
    "RetryPolicyProtocol",
    "SerializerProtocol",
    "TelemetryProtocol",
    "TerminalProtocol",
]
