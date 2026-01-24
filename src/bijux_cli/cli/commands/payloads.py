# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Dataclasses for structured CLI command payloads."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class AuditPayload:
    """Structured payload for audit command results."""

    status: str
    file: str | None = None
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class DoctorPayload:
    """Structured payload for doctor command results."""

    status: str
    summary: list[str]
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class StatusPayload:
    """Structured payload for status command results."""

    status: str
    python: str | None = None
    platform: str | None = None
    ts: float | None = None


@dataclass(frozen=True)
class SleepPayload:
    """Structured payload for sleep command results."""

    slept: float
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class VersionPayload:
    """Structured payload for version command results."""

    version: str
    python: str | None = None
    platform: str | None = None
    timestamp: float | None = None


@dataclass(frozen=True)
class MemorySummaryPayload:
    """Structured payload for memory summary results."""

    status: str
    count: int | None
    message: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class MemoryItemPayload:
    """Structured payload for memory item results."""

    status: str
    key: str
    value: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class MemoryDeletePayload:
    """Structured payload for memory delete results."""

    status: str
    key: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class MemoryListPayload:
    """Structured payload for memory list results."""

    status: str
    keys: list[str]
    count: int
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class MemoryClearPayload:
    """Structured payload for memory clear results."""

    status: str
    count: int
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class HelpPayload:
    """Structured payload for help output."""

    help: str
    python: str | None = None
    platform: str | None = None
    runtime_ms: int | None = None


@dataclass(frozen=True)
class DevDiPayload:
    """Structured payload for dev DI output."""

    factories: list[dict[str, str | None]]
    services: list[dict[str, str | None]]
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class DevPluginsPayload:
    """Structured payload for dev plugin listing."""

    plugins: list[dict[str, object]]
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class DevStatusPayload:
    """Structured payload for dev status."""

    status: str
    mode: str | None = None
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigGetPayload:
    """Structured payload for config get output."""

    value: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigSetPayload:
    """Structured payload for config set output."""

    status: str
    key: str
    value: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigLoadPayload:
    """Structured payload for config load output."""

    status: str
    file: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigClearPayload:
    """Structured payload for config clear output."""

    status: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigReloadPayload:
    """Structured payload for config reload output."""

    status: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigUnsetPayload:
    """Structured payload for config unset output."""

    status: str
    key: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigListPayload:
    """Structured payload for config list output."""

    items: list[dict[str, str]]
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigExportPayload:
    """Structured payload for config export output."""

    status: str
    file: str
    format: str | None = None
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class ConfigDumpPayload:
    """Structured payload for config dump output."""

    entries: dict[str, object]
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class DocsSpecPayload:
    """Structured payload for docs spec output."""

    version: str
    commands: list[str]
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class DocsWritePayload:
    """Structured payload for docs write output."""

    status: str
    file: str


@dataclass(frozen=True)
class HistoryImportPayload:
    """Structured payload for history import output."""

    status: str
    file: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class HistoryExportPayload:
    """Structured payload for history export output."""

    status: str
    file: str
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class HistoryEntriesPayload:
    """Structured payload for history list output."""

    entries: list[dict[str, object]]
    python: str | None = None
    platform: str | None = None


@dataclass(frozen=True)
class HistoryClearPayload:
    """Structured payload for history clear output."""

    status: str
    python: str | None = None
    platform: str | None = None


__all__ = [
    "AuditPayload",
    "DoctorPayload",
    "StatusPayload",
    "SleepPayload",
    "VersionPayload",
    "MemorySummaryPayload",
    "MemoryItemPayload",
    "MemoryDeletePayload",
    "MemoryListPayload",
    "MemoryClearPayload",
    "HelpPayload",
    "DevDiPayload",
    "DevPluginsPayload",
    "DevStatusPayload",
    "ConfigGetPayload",
    "ConfigSetPayload",
    "ConfigLoadPayload",
    "ConfigClearPayload",
    "ConfigReloadPayload",
    "ConfigUnsetPayload",
    "ConfigListPayload",
    "ConfigExportPayload",
    "ConfigDumpPayload",
    "DocsSpecPayload",
    "DocsWritePayload",
    "HistoryImportPayload",
    "HistoryExportPayload",
    "HistoryEntriesPayload",
    "HistoryClearPayload",
]
