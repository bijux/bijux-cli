# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for telemetry resolution."""

from __future__ import annotations

from typing import cast

import pytest

from bijux_cli.core.di import DIContainer
from bijux_cli.infra.telemetry import LoggingTelemetry, NoopTelemetry
from bijux_cli.services.diagnostics.telemetry import resolve_telemetry


class _DI:
    def __init__(self) -> None:
        self.calls: list[type] = []

    def resolve(self, cls: type) -> object:
        self.calls.append(cls)
        return object()


def test_resolve_telemetry_env_enabled(monkeypatch: pytest.MonkeyPatch) -> None:
    di = _DI()
    monkeypatch.setenv("BIJUXCLI_TELEMETRY", "true")
    resolve_telemetry(cast(DIContainer, di), enabled=None)
    assert di.calls == [LoggingTelemetry]


def test_resolve_telemetry_env_disabled(monkeypatch: pytest.MonkeyPatch) -> None:
    di = _DI()
    monkeypatch.setenv("BIJUXCLI_TELEMETRY", "0")
    resolve_telemetry(cast(DIContainer, di), enabled=None)
    assert di.calls == [NoopTelemetry]


def test_resolve_telemetry_explicit_flag() -> None:
    di = _DI()
    resolve_telemetry(cast(DIContainer, di), enabled=True)
    assert di.calls == [LoggingTelemetry]
