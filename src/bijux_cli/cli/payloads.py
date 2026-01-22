# SPDX-License-Identifier: MIT
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
]
