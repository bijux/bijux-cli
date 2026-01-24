# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the plugins info module."""

from __future__ import annotations

from typing import Any

import pytest

import bijux_cli.cli.plugins.commands.info as plugins_info
from bijux_cli.cli.plugins.commands.info import info_plugin
from bijux_cli.core.enums import LogLevel
from bijux_cli.plugins.metadata import PluginMetadata, PluginMetadataError


class DummyExitError(Exception):
    """Capture exit details in tests."""

    def __init__(self, code: int, payload: dict[str, Any]) -> None:
        self.code = code
        self.payload = payload


@pytest.fixture(autouse=True)
def _capture_emit(monkeypatch: pytest.MonkeyPatch) -> None:
    """Intercept error emissions and raise a custom exception."""

    def fake_resolve_exit_intent(
        message: str,
        code: int,
        failure: str,
        **kwargs: Any,
    ) -> None:
        _ = kwargs
        raise DummyExitError(code, {"error": message, "failure": failure})

    monkeypatch.setattr(plugins_info, "resolve_exit_intent", fake_resolve_exit_intent)


def test_info_plugin_not_found(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        plugins_info,
        "get_plugin_metadata",
        lambda _: (_ for _ in ()).throw(PluginMetadataError("not found")),
    )
    with pytest.raises(DummyExitError) as exc:
        info_plugin(
            "foo",
            fmt="json",
            quiet=False,
            pretty=False,
            log_level=LogLevel.INFO,
        )
    assert exc.value.code == 1
    assert exc.value.payload["failure"] == "metadata_error"


def test_info_success(monkeypatch: pytest.MonkeyPatch) -> None:
    meta = PluginMetadata(
        name="foo",
        version="1.0.0",
        enabled=True,
        source="entrypoint",
        requires_cli=">=0.1.0",
        dist_name="foo-plugin",
    )
    monkeypatch.setattr(plugins_info, "get_plugin_metadata", lambda _: meta)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        plugins_info, "new_run_command", lambda **kw: captured.update(kw)
    )

    info_plugin(
        "foo",
        fmt="json",
        quiet=False,
        pretty=True,
        log_level=LogLevel.INFO,
    )

    payload = captured["payload_builder"](False)
    assert payload["name"] == "foo"
    assert payload["version"] == "1.0.0"
    assert payload["enabled"] is True
    assert payload["source"] == "entrypoint"
