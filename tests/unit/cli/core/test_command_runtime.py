# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Tests for runtime helpers in cli.core.command."""

from __future__ import annotations

import errno
from typing import Any, cast

import pytest

from bijux_cli.cli.core import command as cmd
from bijux_cli.cli.core.constants import ENV_CONFIG, ENV_PREFIX
from bijux_cli.core.enums import ErrorType, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.infra.contracts import Serializer


def test_record_history_skips_history_command(monkeypatch: pytest.MonkeyPatch) -> None:
    """History command should not resolve DI."""
    monkeypatch.setattr(
        "bijux_cli.core.di.DIContainer.current",
        staticmethod(lambda: (_ for _ in ()).throw(AssertionError("DI called"))),
    )
    cmd.record_history("history", 0)


def test_record_history_permission_error(capsys: pytest.CaptureFixture[str]) -> None:
    """Permission errors should be reported to stderr."""

    class _Stub:
        def resolve(self, _key: Any) -> Any:
            raise PermissionError("nope")

    with pytest.MonkeyPatch.context() as monkeypatch:
        monkeypatch.setattr(
            "bijux_cli.core.di.DIContainer.current",
            staticmethod(lambda: _Stub()),
        )
        cmd.record_history("status", 1)
    err = capsys.readouterr().err
    assert "Permission denied writing history" in err


def test_record_history_enospc(capsys: pytest.CaptureFixture[str]) -> None:
    """Disk-full errors should be reported clearly."""

    class _Stub:
        def resolve(self, _key: Any) -> Any:
            exc = OSError("full")
            exc.errno = errno.ENOSPC
            raise exc

    with pytest.MonkeyPatch.context() as monkeypatch:
        monkeypatch.setattr(
            "bijux_cli.core.di.DIContainer.current",
            staticmethod(lambda: _Stub()),
        )
        cmd.record_history("status", 1)
    err = capsys.readouterr().err
    assert "No space left on device" in err


def test_record_history_eacces(capsys: pytest.CaptureFixture[str]) -> None:
    """Access denied errors should use the permission message."""

    class _Stub:
        def resolve(self, _key: Any) -> Any:
            exc = OSError("denied")
            exc.errno = errno.EACCES
            raise exc

    with pytest.MonkeyPatch.context() as monkeypatch:
        monkeypatch.setattr(
            "bijux_cli.core.di.DIContainer.current",
            staticmethod(lambda: _Stub()),
        )
        cmd.record_history("status", 1)
    err = capsys.readouterr().err
    assert "Permission denied writing history" in err


def test_record_history_generic_error(capsys: pytest.CaptureFixture[str]) -> None:
    """Generic errors should not crash history recording."""

    class _Stub:
        def resolve(self, _key: Any) -> Any:
            raise RuntimeError("boom")

    with pytest.MonkeyPatch.context() as monkeypatch:
        monkeypatch.setattr(
            "bijux_cli.core.di.DIContainer.current",
            staticmethod(lambda: _Stub()),
        )
        cmd.record_history("status", 1)
    err = capsys.readouterr().err
    assert "Error writing history" in err


def test_raise_exit_intent_rejects_extra_args() -> None:
    with pytest.raises(TypeError):
        cmd.raise_exit_intent("a", "b")


def test_resolve_serializer_requires_dumps(monkeypatch: pytest.MonkeyPatch) -> None:
    class _Stub:
        pass

    monkeypatch.setattr(
        "bijux_cli.core.di.DIContainer.current",
        staticmethod(lambda: type("X", (), {"resolve": lambda *_a, **_k: _Stub()})()),
    )
    with pytest.raises(RuntimeError, match="dumps"):
        cmd.resolve_serializer()


def test_resolve_emitter_returns_none_on_error(monkeypatch: pytest.MonkeyPatch) -> None:
    class _Stub:
        def resolve(self, _key: Any) -> Any:
            raise RuntimeError("fail")

    monkeypatch.setattr(
        "bijux_cli.core.di.DIContainer.current",
        staticmethod(lambda: _Stub()),
    )
    assert cmd.resolve_emitter() is None


def test_contains_non_ascii_env_config(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv(ENV_CONFIG, "bad\u00ff")
    assert cmd.contains_non_ascii_env() is True


def test_contains_non_ascii_env_path_notimplemented(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class _Path:
        def __init__(self, _value: str) -> None:
            raise NotImplementedError("no")

    monkeypatch.setenv(ENV_CONFIG, "ok")
    monkeypatch.setattr(cmd, "Path", _Path)
    assert cmd.contains_non_ascii_env() is False


def test_contains_non_ascii_env_file(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Any
) -> None:
    bad = tmp_path / "bad.env"
    bad.write_text("name=\u00ff\n", encoding="utf-8")
    monkeypatch.setenv(ENV_CONFIG, str(bad))
    assert cmd.contains_non_ascii_env() is True


def test_contains_non_ascii_env_file_ascii(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Any
) -> None:
    ok = tmp_path / "ok.env"
    ok.write_text("name=ok\n", encoding="ascii")
    monkeypatch.setenv(ENV_CONFIG, str(ok))
    assert cmd.contains_non_ascii_env() is False


def test_contains_non_ascii_env_value(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv(f"{ENV_PREFIX}FOO", "bad\u00ff")
    assert cmd.contains_non_ascii_env() is True


def test_validate_env_file_if_present_rejects_invalid(tmp_path: Any) -> None:
    path = tmp_path / "bad.env"
    path.write_text("bad line", encoding="utf-8")
    with pytest.raises(ValueError, match="Malformed line"):
        cmd.validate_env_file_if_present(str(path))


def test_emit_payload_writes_stdout(
    capsys: pytest.CaptureFixture[str],
) -> None:
    class _Serializer:
        def dumps(self, payload: object, *, fmt: object, pretty: bool) -> str:
            _ = (fmt, pretty)
            return f"{payload}\n"

        def dumps_bytes(self, payload: object, *, fmt: object, pretty: bool) -> bytes:
            return self.dumps(payload, fmt=fmt, pretty=pretty).encode("utf-8")

        def loads(self, payload: str, *, fmt: object, pretty: bool) -> object:
            _ = (fmt, pretty)
            return payload

    cmd.emit_payload(
        {"ok": True},
        serializer=cast(Serializer, _Serializer()),
        emitter=None,
        fmt=OutputFormat.JSON,
        pretty=False,
        stream="stdout",
    )
    out = capsys.readouterr().out
    assert "{'ok': True}" in out


def test_raise_exit_intent_with_message() -> None:
    with pytest.raises(ExitIntentError):
        cmd.raise_exit_intent(
            "boom",
            code=1,
            failure="internal",
            command="status",
            fmt=OutputFormat.JSON,
            quiet=False,
            include_runtime=False,
            error_type=ErrorType.INTERNAL,
            log_level=LogLevel.INFO,
        )


def test_validate_common_flags_rejects_unknown_format() -> None:
    with pytest.raises(ExitIntentError):
        cmd.validate_common_flags(
            "extra",
            "status",
            quiet=False,
            include_runtime=False,
            log_level=LogLevel.INFO,
        )
