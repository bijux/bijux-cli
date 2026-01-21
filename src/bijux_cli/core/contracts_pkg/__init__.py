# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the public API for all service and infrastructure contracts.

This module acts as the public facade for the application's service contracts,
which are defined using Python's `Protocol`. It aggregates all individual
protocol definitions from the `bijux_cli.core.contracts` submodules into a single,
stable namespace.

By importing from this module, other parts of the application can depend on
these abstract interfaces without being coupled to the concrete implementation
details or the internal structure of the contracts package.
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
