# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the status command."""

from __future__ import annotations

import json
import signal
import time
from types import SimpleNamespace
from typing import Any

import pytest
import typer

import bijux_cli.cli.commands.status as mod
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ColorMode, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError
from bijux_cli.core.precedence import ExecutionPolicy, resolve_log_policy
from bijux_cli.infra.contracts import Emitter, Serializer
from bijux_cli.services.contracts import TelemetryProtocol


class FakeEmitter(Emitter):
    """Fake emitter."""

    def __init__(self, raise_on_stop: bool = False) -> None:
        """Init."""
        self.calls: list[tuple[Any, dict[str, Any]]] = []
        self.raise_on_stop: bool = raise_on_stop

    def emit(
        self,
        payload: Any,
        *,
        fmt: str | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "Emitting output",
        output: str | None = None,
        **context: Any,
    ) -> None:
        """Record call."""
        if self.raise_on_stop and level == "info":
            raise ValueError("stop emit fail")
        self.calls.append(
            (
                payload,
                {
                    "fmt": fmt,
                    "pretty": pretty,
                    "level": level,
                    "message": message,
                    "output": output,
                    **context,
                },
            )
        )

    def flush(self) -> None:
        """Flush."""
        return None

    def close(self) -> None:
        """Close."""
        return None


class FakeTelemetry(TelemetryProtocol):
    """Fake telemetry."""

    def __init__(self) -> None:
        """Init."""
        self.events: list[tuple[str, dict[str, Any] | None]] = []
        self.enabled: bool = True

    def event(self, name: str, payload: dict[str, Any] | None = None) -> None:
        """Record event."""
        if self.enabled:
            self.events.append((name, payload))

    def enable(self) -> None:
        """Enable."""
        self.enabled = True
        return None

    def disable(self) -> None:
        """Disable."""
        self.enabled = False
        return None

    def flush(self) -> None:
        """Flush."""
        return None


def _payload_status(payload: Any) -> str | None:
    """Return status from payloads that may be dicts or dataclasses."""
    if isinstance(payload, dict):
        return payload.get("status")
    return getattr(payload, "status", None)


class FakeDI:
    """A fake Dependency Injection container for testing."""

    def __init__(self, emitter: Emitter, telemetry: TelemetryProtocol) -> None:
        """Initialize the fake DI container with specific fakes."""
        self._e = emitter
        self._t = telemetry
        self._s = _BasicSerializer()

    def resolve(self, key: Any) -> Emitter | TelemetryProtocol:
        """Resolve a dependency to its fake implementation."""
        if key is Emitter:
            return self._e
        if key is TelemetryProtocol:
            return self._t
        if key is Serializer:
            return self._s
        raise KeyError(key)


class _BasicSerializer:
    """Minimal JSON serializer for tests."""

    def dumps(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> str:
        _ = fmt
        return json.dumps(obj, indent=2 if pretty else None)

    def dumps_bytes(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> bytes:
        return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

    def loads(self, data: str | bytes, *, fmt: OutputFormat, pretty: bool) -> Any:
        _ = (fmt, pretty)
        if isinstance(data, bytes):
            data = data.decode("utf-8")
        return json.loads(data)

    def emit(self, payload: Any, *, fmt: OutputFormat, pretty: bool) -> None:
        _ = (payload, fmt, pretty)
        return None


def test_build_payload_minimal(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test building the minimal status payload without runtime info."""
    called = {"ascii": 0}

    def _fake_ascii(v: Any, n: str) -> None:
        called["ascii"] += 1

    monkeypatch.setattr(mod, "ascii_safe", _fake_ascii)
    p = mod._build_payload(include_runtime=False)
    assert p["status"] == "ok"
    assert called["ascii"] == 0


def test_build_payload_with_runtime(monkeypatch: pytest.MonkeyPatch) -> None:
    """Build payload with runtime info."""

    def fake_ascii_safe(v: str, n: str) -> str:
        return v

    monkeypatch.setattr(mod, "ascii_safe", fake_ascii_safe)
    p = mod._build_payload(include_runtime=True)
    assert p["status"] == "ok"
    assert isinstance(p["python"], str)
    assert isinstance(p["platform"], str)


def test_run_watch_mode_rejects_non_json(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that watch mode rejects non-JSON output formats."""
    seen: dict[str, Any] = {}

    def fake_exit(**kwargs: Any) -> ExitIntent:
        seen.update(kwargs)
        return ExitIntent(
            code=ExitCode(kwargs["code"]),
            stream="stderr",
            payload={"error": kwargs["message"]},
            fmt=kwargs["fmt"],
            pretty=False,
            show_traceback=False,
        )

    monkeypatch.setattr(mod, "resolve_exit_intent", fake_exit)
    em, tel = FakeEmitter(), FakeTelemetry()
    with pytest.raises(ExitIntentError) as ei:
        mod._run_watch_mode(
            command="status",
            watch_interval=0.01,
            fmt=OutputFormat.YAML,
            quiet=False,
            effective_pretty=True,
            include_runtime=False,
            log_policy=resolve_log_policy(LogLevel.INFO),
            telemetry=tel,
            emitter=em,
        )
    assert ei.value.intent.code == ExitCode.USAGE
    assert seen["failure"] == "watch_fmt"


def test_run_watch_mode_ascii_value_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that watch mode handles ascii_safe errors correctly."""

    def bad_ascii(*_a: Any, **_k: Any) -> None:
        raise ValueError("bad ascii")

    monkeypatch.setattr(mod, "ascii_safe", bad_ascii)

    def fake_exit(**kwargs: Any) -> ExitIntent:
        assert kwargs["failure"] == "ascii"
        return ExitIntent(
            code=ExitCode(kwargs["code"]),
            stream="stderr",
            payload={"error": kwargs["message"]},
            fmt=kwargs["fmt"],
            pretty=False,
            show_traceback=False,
        )

    monkeypatch.setattr(mod, "resolve_exit_intent", fake_exit)
    em, tel = FakeEmitter(), FakeTelemetry()
    with pytest.raises(ExitIntentError) as ei:
        mod._run_watch_mode(
            command="status",
            watch_interval=0.0,
            fmt=OutputFormat.JSON,
            quiet=False,
            effective_pretty=True,
            include_runtime=True,
            log_policy=resolve_log_policy(LogLevel.INFO),
            telemetry=tel,
            emitter=em,
        )
    assert ei.value.intent.code == ExitCode.ASCII


def test_run_watch_mode_generic_emit_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that watch mode handles generic emitter errors."""

    class BoomEmitter(FakeEmitter):
        def emit(self, *a: Any, **k: Any) -> None:
            raise RuntimeError("boom")

    em, tel = BoomEmitter(), FakeTelemetry()

    def fake_exit(**kwargs: Any) -> ExitIntent:
        assert kwargs["failure"] == "emit"
        return ExitIntent(
            code=ExitCode(kwargs["code"]),
            stream="stderr",
            payload={"error": kwargs["message"]},
            fmt=kwargs["fmt"],
            pretty=False,
            show_traceback=False,
        )

    monkeypatch.setattr(mod, "resolve_exit_intent", fake_exit)
    with pytest.raises(ExitIntentError) as ei:
        mod._run_watch_mode(
            command="status",
            watch_interval=0.0,
            fmt=OutputFormat.JSON,
            quiet=False,
            effective_pretty=True,
            include_runtime=False,
            log_policy=resolve_log_policy(LogLevel.INFO),
            telemetry=tel,
            emitter=em,
        )
    assert ei.value.intent.code == ExitCode.ERROR


def test_status_returns_early_on_subcommand(monkeypatch: pytest.MonkeyPatch) -> None:
    """Exit early when a subcommand is invoked."""
    from typing import cast

    called: dict[str, int] = {"new_run": 0, "watch": 0}

    def _fake_new_run(**kw: Any) -> None:
        called["new_run"] += 1

    def _fake_watch(**kw: Any) -> None:
        called["watch"] += 1

    def _validate(fmt: str, *_a: Any, **_k: Any) -> str:
        return fmt.lower()

    monkeypatch.setattr(mod, "new_run_command", _fake_new_run)
    monkeypatch.setattr(mod, "_run_watch_mode", _fake_watch)
    monkeypatch.setattr(mod, "validate_common_flags", _validate)

    ctx = cast(typer.Context, SimpleNamespace(invoked_subcommand="other"))
    mod.status(
        ctx,
        watch=None,
        quiet=False,
        fmt=OutputFormat.JSON,
        pretty=True,
        log_level=LogLevel.INFO,
    )
    assert called["new_run"] == 0
    assert called["watch"] == 0


def test_status_calls_new_run_command_when_not_watching(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Delegate to new_run_command in non-watch mode."""
    from typing import cast

    em, tel = FakeEmitter(), FakeTelemetry()
    monkeypatch.setattr(DIContainer, "current", lambda: FakeDI(em, tel))
    monkeypatch.setattr(
        "bijux_cli.cli.commands.status.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=True,
            log_level=LogLevel.INFO,
            pretty=False,
            include_runtime=True,
        ),
    )
    seen: dict[str, Any] = {}

    def fake_new_run_command(**kw: Any) -> None:
        seen.update(kw)

    def _validate(fmt: str, *_a: Any, **_k: Any) -> str:
        return fmt.lower()

    monkeypatch.setattr(mod, "new_run_command", fake_new_run_command)
    monkeypatch.setattr(mod, "validate_common_flags", _validate)

    ctx = cast(typer.Context, SimpleNamespace(invoked_subcommand=None))
    mod.status(
        ctx,
        watch=None,
        quiet=True,
        fmt="JSON",
        pretty=False,
        log_level=LogLevel.INFO,
    )
    assert seen["command_name"] == "status"
    assert seen["quiet"] is True
    assert seen["fmt"] == "json"
    assert seen["pretty"] is False
    pb = seen["payload_builder"]
    payload = pb(True)
    assert payload["status"] == "ok"
    assert payload["python"]
    assert payload["platform"]


def test_status_watch_invalid_interval_types(monkeypatch: pytest.MonkeyPatch) -> None:
    """Error on invalid watch interval types or values."""
    from typing import cast

    def _validate(fmt: str, *_a: Any, **_k: Any) -> str:
        return fmt.lower()

    monkeypatch.setattr(mod, "validate_common_flags", _validate)
    monkeypatch.setattr(
        DIContainer, "current", lambda: FakeDI(FakeEmitter(), FakeTelemetry())
    )

    def fake_exit(**kwargs: Any) -> ExitIntent:
        assert kwargs["failure"] == "interval"
        return ExitIntent(
            code=ExitCode(kwargs["code"]),
            stream="stderr",
            payload={"error": kwargs["message"]},
            fmt=kwargs["fmt"],
            pretty=False,
            show_traceback=False,
        )

    monkeypatch.setattr(mod, "resolve_exit_intent", fake_exit)
    ctx = cast(typer.Context, SimpleNamespace(invoked_subcommand=None))

    with pytest.raises(typer.Exit) as e1:
        mod.status(
            ctx,
            watch=0,
            quiet=False,
            fmt=OutputFormat.JSON,
            pretty=True,
            log_level=LogLevel.INFO,
        )
    assert e1.value.exit_code == ExitCode.USAGE

    with pytest.raises(typer.Exit) as e2:
        mod.status(
            ctx,
            watch=cast(Any, "abc"),
            quiet=False,
            fmt=OutputFormat.JSON,
            pretty=True,
            log_level=LogLevel.INFO,
        )
    assert e2.value.exit_code == ExitCode.USAGE


def test_status_watch_happy_path_delegates_to_run_watch_mode(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Delegates to _run_watch_mode in watch mode."""
    from typing import cast

    em, tel = FakeEmitter(), FakeTelemetry()
    monkeypatch.setattr(DIContainer, "current", lambda: FakeDI(em, tel))

    def _validate(fmt: str, *_a: Any, **_k: Any) -> str:
        return fmt.lower()

    monkeypatch.setattr(mod, "validate_common_flags", _validate)
    monkeypatch.setattr(
        "bijux_cli.cli.commands.status.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=True,
            log_level=LogLevel.DEBUG,
            pretty=True,
            include_runtime=False,
        ),
    )
    seen: dict[str, Any] = {}

    def fake_run_watch_mode(**kw: Any) -> None:
        seen.update(kw)

    monkeypatch.setattr(mod, "_run_watch_mode", fake_run_watch_mode)
    ctx = cast(typer.Context, SimpleNamespace(invoked_subcommand=None))
    mod.status(
        ctx,
        watch=0.5,
        quiet=True,
        fmt="JSON",
        pretty=True,
        log_level=LogLevel.DEBUG,
    )
    assert seen["command"] == "status"
    assert seen["watch_interval"] == pytest.approx(0.5)
    assert seen["fmt"] == "json"
    assert seen["quiet"] is True
    assert seen["include_runtime"] is False
    assert seen["log_policy"].level == LogLevel.DEBUG
    assert seen["effective_pretty"] is True
    assert seen["telemetry"] is tel
    assert seen["emitter"] is em


def test_run_watch_mode_quiet_skips_final_emit_but_records_stop(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Quiet mode skips emits but records telemetry."""
    em = FakeEmitter()
    tel = FakeTelemetry()

    def _sleep(_s: float) -> None:
        signal.raise_signal(signal.SIGINT)

    monkeypatch.setattr(time, "sleep", _sleep)
    mod._run_watch_mode(
        command="status",
        watch_interval=0.01,
        fmt=OutputFormat.JSON,
        quiet=True,
        effective_pretty=True,
        include_runtime=False,
        log_policy=resolve_log_policy(LogLevel.INFO),
        telemetry=tel,
        emitter=em,
    )
    assert em.calls == []
    names = [n for n, _ in tel.events]
    assert "COMMAND_SUCCESS" in names
    assert "COMMAND_STOPPED" in names


def test_run_watch_mode_one_iteration_and_stop(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """One iteration then SIGINT stop."""
    em = FakeEmitter()
    tel = FakeTelemetry()

    def sleep_then_sigint(_secs: float) -> None:
        signal.raise_signal(signal.SIGINT)

    monkeypatch.setattr(time, "sleep", sleep_then_sigint)
    mod._run_watch_mode(
        command="status",
        watch_interval=0.01,
        fmt=OutputFormat.JSON,
        quiet=False,
        effective_pretty=True,
        include_runtime=True,
        log_policy=resolve_log_policy(LogLevel.DEBUG),
        telemetry=tel,
        emitter=em,
    )
    assert any(call[1]["level"] == "info" for call in em.calls)
    assert any(
        call[1]["level"] == "info" and _payload_status(call[0]) == "watch-stopped"
        for call in em.calls
    )
    names = [n for n, _ in tel.events]
    assert "COMMAND_SUCCESS" in names
    assert "COMMAND_STOPPED" in names
    assert any(call[1]["level"] == LogLevel.DEBUG for call in em.calls)


def test_run_watch_mode_info_suppresses_diagnostics(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """INFO log level should not emit internal diagnostics."""
    em = FakeEmitter()
    tel = FakeTelemetry()

    def sleep_then_sigint(_secs: float) -> None:
        signal.raise_signal(signal.SIGINT)

    monkeypatch.setattr(time, "sleep", sleep_then_sigint)
    mod._run_watch_mode(
        command="status",
        watch_interval=0.01,
        fmt=OutputFormat.JSON,
        quiet=False,
        effective_pretty=True,
        include_runtime=False,
        log_policy=resolve_log_policy(LogLevel.INFO),
        telemetry=tel,
        emitter=em,
    )
    assert em.calls
    assert all(call[1].get("emit_diagnostics") is False for call in em.calls)
    assert all(call[1].get("level") != LogLevel.DEBUG for call in em.calls)


def test_run_watch_mode_final_emit_exception_swallowed(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Swallow final emit exception."""
    em = FakeEmitter()
    tel = FakeTelemetry()

    def _sleep(_s: float) -> None:
        signal.raise_signal(signal.SIGINT)

    def raising_emit(
        payload: Any,
        *,
        fmt: str | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "Emitting output",
        output: str | None = None,
        **context: Any,
    ) -> None:
        if _payload_status(payload) == "watch-stopped":
            raise ValueError("stop emit fail")
        return FakeEmitter.emit(
            em,
            payload,
            fmt=fmt,
            pretty=pretty,
            level=level,
            message=message,
            output=output,
            **context,
        )

    monkeypatch.setattr(time, "sleep", _sleep)
    monkeypatch.setattr(em, "emit", raising_emit)
    mod._run_watch_mode(
        command="status",
        watch_interval=0.0,
        fmt=OutputFormat.JSON,
        quiet=False,
        effective_pretty=False,
        include_runtime=False,
        log_policy=resolve_log_policy(LogLevel.INFO),
        telemetry=tel,
        emitter=em,
    )
    assert not any(n == "COMMAND_STOPPED" for n, _ in tel.events)
    assert any(n == "COMMAND_SUCCESS" for n, _ in tel.events)
