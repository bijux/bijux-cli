# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Defines contracts for diagnostics services."""

from __future__ import annotations

from collections.abc import Mapping
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

    def log(self, cmd: list[str], *, executor: str) -> None:
        """Record a command execution."""
        ...

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]:
        """Run a command through the audit service."""
        ...

    def cli_audit(self) -> None:
        """Run an audit of the CLI environment."""
        ...

    def shutdown(self) -> None:
        """Shutdown the audit service."""
        ...


@runtime_checkable
class DocsProtocol(Protocol):
    """Defines the contract for documentation generation."""

    def render(self, spec: Mapping[str, Any], *, fmt: Any, pretty: bool = False) -> str:
        """Render a document spec to a string."""
        ...

    def write(
        self,
        spec: Mapping[str, Any],
        *,
        fmt: Any,
        name: str,
        pretty: bool = False,
    ) -> str:
        """Write a document spec to disk."""
        ...


@runtime_checkable
class DoctorProtocol(Protocol):
    """Defines the contract for health checks."""

    def check_health(self) -> str:
        """Run health checks and return a summary."""
        ...


@runtime_checkable
class MemoryProtocol(Protocol):
    """Defines the contract for key-value memory storage."""

    def get(self, key: str) -> Any:
        """Retrieve a stored value."""
        ...

    def set(self, key: str, value: Any) -> None:
        """Store a key-value pair."""
        ...

    def delete(self, key: str) -> None:
        """Delete a stored value."""
        ...

    def clear(self) -> None:
        """Clear all stored values."""
        ...

    def keys(self) -> list[str]:
        """List stored keys."""
        ...


__all__ = [
    "AuditProtocol",
    "DocsProtocol",
    "DoctorProtocol",
    "MemoryProtocol",
    "DiagnosticsConfig",
]
