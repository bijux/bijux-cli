# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines contracts for diagnostics services."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Protocol, runtime_checkable


@dataclass(frozen=True)
class DiagnosticsConfig:
    """Configuration for diagnostics services."""

    enabled: bool
    telemetry_enabled: bool


@runtime_checkable
class AuditProtocol(Protocol):
    """Defines the contract for auditing command execution."""

    def log(self, cmd: list[str], *, executor: str) -> None: ...

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]: ...

    def cli_audit(self) -> None: ...

    def shutdown(self) -> None: ...


@runtime_checkable
class DocsProtocol(Protocol):
    """Defines the contract for documentation generation."""

    def render(self, spec: dict[str, Any], *, fmt: Any, pretty: bool = False) -> str: ...

    def write(
        self, spec: dict[str, Any], *, fmt: Any, name: str, pretty: bool = False
    ) -> str: ...


@runtime_checkable
class DoctorProtocol(Protocol):
    """Defines the contract for health checks."""

    def check_health(self) -> str: ...


@runtime_checkable
class MemoryProtocol(Protocol):
    """Defines the contract for key-value memory storage."""

    def get(self, key: str) -> Any: ...

    def set(self, key: str, value: Any) -> None: ...

    def delete(self, key: str) -> None: ...

    def clear(self) -> None: ...

    def list(self) -> dict[str, Any]: ...


__all__ = [
    "AuditProtocol",
    "DocsProtocol",
    "DoctorProtocol",
    "MemoryProtocol",
    "DiagnosticsConfig",
]
