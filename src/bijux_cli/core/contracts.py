# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Protocol and contract definitions (facade).

Contracts in this module are application-wide interfaces used by core and
services. Service-local interfaces belong in `services/<domain>/contracts.py`
and are not promoted into the core contract surface.

Contract ownership (annotated):
- ContextProtocol: core behavioral
- ObservabilityProtocol: core behavioral (used by core)
- RegistryProtocol: core behavioral (plugin registry)
- EmitterProtocol: infra-facing
- FileSystemProtocol: infra-facing
- ProcessPoolProtocol: infra-facing
- RetryPolicyProtocol: infra-facing
- SerializerProtocol: infra-facing
- TelemetryProtocol: infra-facing
- TerminalProtocol: infra-facing
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
    # Core behavioral
    "ContextProtocol",
    "ObservabilityProtocol",
    "RegistryProtocol",
    # Infra-facing
    "EmitterProtocol",
    "FileSystemProtocol",
    "ProcessPoolProtocol",
    "RetryPolicyProtocol",
    "SerializerProtocol",
    "TelemetryProtocol",
    "TerminalProtocol",
]
