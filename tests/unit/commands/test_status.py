# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the status command."""

from __future__ import annotations

import signal
import time
from types import SimpleNamespace
from typing import Any

import pytest
import typer

import bijux_cli.commands.status as mod
from bijux_cli.contracts import EmitterProtocol, TelemetryProtocol
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import OutputFormat


class FakeEmitter(EmitterProtocol):
    """Fake emitter."""

    def __init__(self, raise_on_stop: bool = False) -> None:
        """Init."""
        self.calls: list[tuple[Any, dict[str, Any]]] = []
        self.raise_on_stop: bool = raise_on_stop

    def emit(
        self,
        payload: Any,
        *,
        fmt: OutputFormat | None = None,
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


class FakeDI:
    """A fake Dependency Injection container for testing."""

    def __init__(self, emitter: EmitterProtocol, telemetry: TelemetryProtocol) -> None:
        """Initialize the fake DI container with specific fakes."""
        self._e = emitter
        self._t = telemetry

    def resolve(self, key: Any) -> EmitterProtocol | TelemetryProtocol:
        """Resolve a dependency to its fake implementation."""
        if key is EmitterProtocol:
            return self._e
        if key is TelemetryProtocol:
            return self._t
        raise KeyError(key)


def test_build_payload_minimal(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test building the minimal status payload without runtime info."""
    called = {"ascii": 0}

    def _fake_ascii(v: Any, n: str) -> None:
        called["ascii"] += 1

    monkeypatch.setattr(mod, "ascii_safe", _fake_ascii)
    p = mod._build_payload(include_runtime=False)  # pyright: ignore[reportPrivateUsage]
    assert p == {"status": "ok"}
    assert called["ascii"] == 0


def test_build_payload_with_runtime(monkeypatch: pytest.MonkeyPatch) -> None:
    """Build payload with runtime info."""
    from collections.abc import Mapping

    def fake_ascii_safe(v: str, n: str) -> str:
        return v

    monkeypatch.setattr(mod, "ascii_safe", fake_ascii_safe)
    p: Mapping[str, object] = mod._build_payload(  # pyright: ignore[reportPrivateUsage]
        include_runtime=True
    )
    assert p["status"] == "ok"
    assert isinstance(p["python"], str)
    assert isinstance(p["platform"], str)


def test_run_watch_mode_rejects_non_json(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that watch mode rejects non-JSON output formats."""
    seen: dict[str, Any] = {}

    def fake_exit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        **kw: Any,
    ) -> None:
        seen.update(locals())
        raise SystemExit(code)

    monkeypatch.setattr(mod, "emit_error_and_exit", fake_exit)
    em, tel = FakeEmitter(), FakeTelemetry()
    with pytest.raises(SystemExit) as ei:
        mod._run_watch_mode(  # pyright: ignore[reportPrivateUsage]
            command="status",
            watch_interval=0.01,
            fmt="yaml",
            quiet=False,
            verbose=False,
            debug=False,
            effective_pretty=True,
            include_runtime=False,
            telemetry=tel,
            emitter=em,
        )
    assert ei.value.code == 2
    assert seen["failure"] == "watch_fmt"


def test_run_watch_mode_ascii_value_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that watch mode handles ascii_safe errors correctly."""

    def bad_ascii(*_a: Any, **_k: Any) -> None:
        raise ValueError("bad ascii")

    monkeypatch.setattr(mod, "ascii_safe", bad_ascii)

    def fake_exit(msg: str, code: int, failure: str, **_: Any) -> None:
        assert failure == "ascii"
        raise SystemExit(code)

    monkeypatch.setattr(mod, "emit_error_and_exit", fake_exit)
    em, tel = FakeEmitter(), FakeTelemetry()
    with pytest.raises(SystemExit) as ei:
        mod._run_watch_mode(  # pyright: ignore[reportPrivateUsage]
            command="status",
            watch_interval=0.0,
            fmt="json",
            quiet=False,
            verbose=True,
            debug=False,
            effective_pretty=True,
            include_runtime=True,
            telemetry=tel,
            emitter=em,
        )
    assert ei.value.code == 3


def test_run_watch_mode_generic_emit_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that watch mode handles generic emitter errors."""

    class BoomEmitter(FakeEmitter):
        def emit(self, *a: Any, **k: Any) -> None:
            raise RuntimeError("boom")

    em, tel = BoomEmitter(), FakeTelemetry()

    def fake_exit(msg: str, code: int, failure: str, **_: Any) -> None:
        assert failure == "emit"
        raise SystemExit(code)

    monkeypatch.setattr(mod, "emit_error_and_exit", fake_exit)
    with pytest.raises(SystemExit) as ei:
        mod._run_watch_mode(  # pyright: ignore[reportPrivateUsage]
            command="status",
            watch_interval=0.0,
            fmt="json",
            quiet=False,
            verbose=False,
            debug=False,
            effective_pretty=True,
            include_runtime=False,
            telemetry=tel,
            emitter=em,
        )
    assert ei.value.code == 1


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
        verbose=False,
        fmt="json",
        pretty=True,
        debug=False,
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
        verbose=True,
        fmt="JSON",
        pretty=False,
        debug=False,
    )
    assert seen["command_name"] == "status"
    assert seen["quiet"] is True
    assert seen["verbose"] is True
    assert seen["fmt"] == "json"
    assert seen["pretty"] is False
    pb = seen["payload_builder"]
    payload = pb(True)
    assert payload["status"] == "ok"
    assert "python" in payload
    assert "platform" in payload


def test_status_watch_invalid_interval_types(monkeypatch: pytest.MonkeyPatch) -> None:
    """Error on invalid watch interval types or values."""
    from typing import cast

    def _validate(fmt: str, *_a: Any, **_k: Any) -> str:
        return fmt.lower()

    monkeypatch.setattr(mod, "validate_common_flags", _validate)
    monkeypatch.setattr(
        DIContainer, "current", lambda: FakeDI(FakeEmitter(), FakeTelemetry())
    )

    def fake_exit(msg: str, code: int, failure: str, **_: Any) -> None:
        assert failure == "interval"
        raise SystemExit(code)

    monkeypatch.setattr(mod, "emit_error_and_exit", fake_exit)
    ctx = cast(typer.Context, SimpleNamespace(invoked_subcommand=None))

    with pytest.raises(SystemExit) as e1:
        mod.status(
            ctx,
            watch=0,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
        )
    assert e1.value.code == 2

    with pytest.raises(SystemExit) as e2:
        mod.status(
            ctx,
            watch=cast(Any, "abc"),
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
        )
    assert e2.value.code == 2


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
    seen: dict[str, Any] = {}

    def fake_run_watch_mode(**kw: Any) -> None:
        seen.update(kw)

    monkeypatch.setattr(mod, "_run_watch_mode", fake_run_watch_mode)
    ctx = cast(typer.Context, SimpleNamespace(invoked_subcommand=None))
    mod.status(
        ctx,
        watch=0.5,
        quiet=True,
        verbose=False,
        fmt="JSON",
        pretty=True,
        debug=True,
    )
    assert seen["command"] == "status"
    assert seen["watch_interval"] == pytest.approx(0.5)
    assert seen["fmt"] == "json"
    assert seen["quiet"] is True
    assert seen["verbose"] is False
    assert seen["debug"] is True
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
    mod._run_watch_mode(  # pyright: ignore[reportPrivateUsage]
        command="status",
        watch_interval=0.01,
        fmt="json",
        quiet=True,
        verbose=False,
        debug=False,
        effective_pretty=True,
        include_runtime=False,
        telemetry=tel,
        emitter=em,
    )
    assert em.calls == []
    names = [n for n, _ in tel.events]
    assert "COMMAND_SUCCESS" in names
    assert "COMMAND_STOPPED" in names


def test_run_watch_mode_one_iteration_and_stop(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """One iteration then SIGINT stop."""
    em = FakeEmitter()
    tel = FakeTelemetry()

    def sleep_then_sigint(_secs: float) -> None:
        signal.raise_signal(signal.SIGINT)

    monkeypatch.setattr(time, "sleep", sleep_then_sigint)
    mod._run_watch_mode(  # pyright: ignore[reportPrivateUsage]
        command="status",
        watch_interval=0.01,
        fmt="json",
        quiet=False,
        verbose=True,
        debug=True,
        effective_pretty=True,
        include_runtime=True,
        telemetry=tel,
        emitter=em,
    )
    assert any(call[1]["level"] == "info" for call in em.calls)
    assert any(
        call[1]["level"] == "info" and call[0].get("status") == "watch-stopped"
        for call in em.calls
    )
    names = [n for n, _ in tel.events]
    assert "COMMAND_SUCCESS" in names
    assert "COMMAND_STOPPED" in names
    err = capsys.readouterr().err
    assert "Debug: Emitting payload" in err
    assert "Debug: Emitting watch-stopped payload" in err


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
        fmt: OutputFormat | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "Emitting output",
        output: str | None = None,
        **context: Any,
    ) -> None:
        if isinstance(payload, dict) and payload.get("status") == "watch-stopped":
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
    mod._run_watch_mode(  # pyright: ignore[reportPrivateUsage]
        command="status",
        watch_interval=0.0,
        fmt="json",
        quiet=False,
        verbose=False,
        debug=False,
        effective_pretty=False,
        include_runtime=False,
        telemetry=tel,
        emitter=em,
    )
    assert not any(n == "COMMAND_STOPPED" for n, _ in tel.events)
    assert any(n == "COMMAND_SUCCESS" for n, _ in tel.events)
