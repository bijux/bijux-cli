# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Regression coverage for bootstrap fast vs runtime paths."""

from __future__ import annotations

from pathlib import Path

import pytest

from bijux_cli.core import bootstrap_flow
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.intent import CLIIntent
from bijux_cli.core.precedence import FlagError, Flags, resolve_log_policy


def _isolate_env(monkeypatch: pytest.MonkeyPatch, root: Path) -> None:
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(root / ".env"))
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(root / "plugins"))
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(root / ".history"))
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "1")


def test_fast_version_skips_di_init(monkeypatch: pytest.MonkeyPatch) -> None:
    called = {"default": 0, "plugins": 0, "engine": 0}

    def _mark_default(*_args: object, **_kwargs: object) -> None:
        called["default"] += 1

    def _mark_plugins(*_args: object, **_kwargs: object) -> None:
        called["plugins"] += 1

    def _mark_engine(*_args: object, **_kwargs: object) -> None:
        called["engine"] += 1

    monkeypatch.setattr(bootstrap_flow, "register_default_services", _mark_default)
    monkeypatch.setattr(bootstrap_flow, "register_plugin_services", _mark_plugins)
    monkeypatch.setattr(bootstrap_flow, "Engine", _mark_engine)
    monkeypatch.setattr(
        bootstrap_flow.sys,
        "argv",
        ["bijux", "version"],
    )

    exit_code = bootstrap_flow.main()
    assert exit_code == 0
    assert called == {"default": 0, "plugins": 0, "engine": 0}


def test_runtime_path_initializes_di(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    called = {"default": 0, "plugins": 0, "engine": 0}
    original_default = bootstrap_flow.register_default_services
    original_plugins = bootstrap_flow.register_plugin_services
    original_engine = bootstrap_flow.Engine

    def _mark_default(*_args: object, **_kwargs: object) -> None:
        called["default"] += 1
        original_default(*_args, **_kwargs)  # type: ignore[arg-type]

    def _mark_plugins(*_args: object, **_kwargs: object) -> None:
        called["plugins"] += 1
        original_plugins(*_args, **_kwargs)  # type: ignore[arg-type]

    def _mark_engine(*_args: object, **_kwargs: object) -> None:
        called["engine"] += 1
        original_engine(*_args, **_kwargs)  # type: ignore[arg-type]

    _isolate_env(monkeypatch, tmp_path)
    monkeypatch.setenv("HOME", str(tmp_path / "home"))
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "xdg_config"))
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path / "xdg_cache"))
    monkeypatch.setenv("XDG_DATA_HOME", str(tmp_path / "xdg_data"))

    monkeypatch.setattr(
        bootstrap_flow.sys,
        "argv",
        ["bijux", "status"],
    )

    # Bind wrappers with access to originals.
    monkeypatch.setattr(bootstrap_flow, "register_default_services", _mark_default)
    monkeypatch.setattr(bootstrap_flow, "register_plugin_services", _mark_plugins)
    monkeypatch.setattr(bootstrap_flow, "Engine", _mark_engine)

    exit_code = bootstrap_flow.main()
    assert exit_code in (0, 1, 2)
    assert called["default"] >= 1
    assert called["plugins"] >= 1
    assert called["engine"] >= 1


def test_intent_error_short_circuits_runtime(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    err = FlagError(
        message="bad flag",
        failure="invalid_flag",
        flag="--bad",
    )
    intent = CLIIntent(
        command=None,
        args=("--bad",),
        flags=Flags(
            quiet=False,
            log_level=LogLevel.INFO,
            color=ColorMode.AUTO,
            format=OutputFormat.JSON,
        ),
        output_format=OutputFormat.JSON,
        log_level=LogLevel.INFO,
        quiet=False,
        color=ColorMode.AUTO,
        pretty=True,
        include_runtime=False,
        log_policy=resolve_log_policy(LogLevel.INFO),
        help=False,
        errors=(err,),
    )
    monkeypatch.setattr(bootstrap_flow, "build_cli_intent", lambda *_a, **_k: intent)
    monkeypatch.setattr(bootstrap_flow, "_emit_fast_error", lambda *_a, **_k: 2)
    monkeypatch.setattr(
        bootstrap_flow,
        "run_runtime",
        lambda _intent: pytest.fail("runtime should not execute on intent errors"),
    )
    monkeypatch.setattr(bootstrap_flow.sys, "argv", ["bijux", "--bad"])
    assert bootstrap_flow.main() == 2


def test_policy_init_failure_returns_internal_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def _boom(*_args: object, **_kwargs: object) -> None:
        raise RuntimeError("boom")

    monkeypatch.setattr(bootstrap_flow, "setup_structlog", _boom)
    monkeypatch.setattr(bootstrap_flow.sys, "argv", ["bijux", "status"])
    exit_code = bootstrap_flow.main()
    assert exit_code == 1


def test_dispatch_failure_emits_error(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    _isolate_env(monkeypatch, tmp_path)
    monkeypatch.setattr(bootstrap_flow.sys, "argv", ["bijux", "status"])

    class _Boom:
        def __call__(self, *args: object, **kwargs: object) -> None:
            raise RuntimeError("boom")

    emitted: dict[str, object] = {}
    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *args, **kwargs: _Boom())
    monkeypatch.setattr(bootstrap_flow, "resolve_serializer", lambda: object())
    monkeypatch.setattr(bootstrap_flow, "resolve_emitter", lambda: object())

    def _emit_payload(payload: object, **_kwargs: object) -> None:
        emitted["payload"] = payload

    monkeypatch.setattr(bootstrap_flow, "emit_payload", _emit_payload)

    exit_code = bootstrap_flow.main()
    assert exit_code == 1
    assert emitted["payload"]
