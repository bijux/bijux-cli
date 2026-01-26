# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit coverage for bootstrap flow runtime paths."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, replace
from types import SimpleNamespace
from typing import Any, cast

from click.exceptions import NoSuchOption, UsageError
import pytest
import typer

from bijux_cli.cli.core.constants import ENV_TEST_MODE
from bijux_cli.core import bootstrap_flow
from bijux_cli.core.enums import ColorMode, ErrorType, LogLevel, OutputFormat
from bijux_cli.core.errors import UserInputError
from bijux_cli.core.intent import CLIIntent
from bijux_cli.core.precedence import FlagError, Flags, resolve_log_policy
from bijux_cli.services.history import History


@dataclass
class _DummyHistory:
    entries: list[dict[str, Any]]

    def add(self, **kwargs: Any) -> None:
        self.entries.append(kwargs)


class _DummyContainer:
    def __init__(self, history: _DummyHistory) -> None:
        self._history = history

    def register(self, *_args: object, **_kwargs: object) -> None:
        return None

    def resolve(self, service: object) -> object:
        if service is History:
            return self._history
        raise KeyError(service)


def _intent() -> CLIIntent:
    policy = resolve_log_policy(LogLevel.INFO)
    return CLIIntent(
        command="status",
        args=("status",),
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
        pretty=policy.pretty_default,
        include_runtime=policy.show_internal,
        log_policy=policy,
        help=False,
        errors=(),
    )


def test_emit_fast_payload_json(capsys: pytest.CaptureFixture[str]) -> None:
    bootstrap_flow._emit_fast_payload(
        {"ok": True}, fmt=OutputFormat.JSON, stream="stdout"
    )
    assert '"ok": true' in capsys.readouterr().out


def test_emit_fast_payload_yaml(capsys: pytest.CaptureFixture[str]) -> None:
    bootstrap_flow._emit_fast_payload(
        {"ok": True}, fmt=OutputFormat.YAML, stream="stdout"
    )
    assert "ok" in capsys.readouterr().out


def test_emit_fast_payload_dataclass(capsys: pytest.CaptureFixture[str]) -> None:
    @dataclass
    class _Payload:
        status: str

    bootstrap_flow._emit_fast_payload(
        _Payload(status="ok"), fmt=OutputFormat.JSON, stream="stdout"
    )
    assert '"status": "ok"' in capsys.readouterr().out


def test_emit_fast_payload_yaml_missing(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    import builtins

    real_import = cast(Callable[..., object], builtins.__import__)

    def fake_import(name: str, *args: object, **kwargs: object) -> object:
        if name == "yaml":
            raise ImportError
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", fake_import)
    bootstrap_flow._emit_fast_payload(
        {"ok": True}, fmt=OutputFormat.YAML, stream="stdout"
    )
    assert '"ok": true' in capsys.readouterr().out


def test_emit_fast_error_quiet() -> None:
    code = bootstrap_flow._emit_fast_error(
        "nope",
        error_type=ErrorType.CONFIG,
        quiet=True,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
    )
    assert code == 1


def test_emit_fast_error_no_stream(monkeypatch: pytest.MonkeyPatch) -> None:
    from bijux_cli.core.enums import ExitCode
    from bijux_cli.core.exit_policy import ExitBehavior

    behavior = ExitBehavior(ExitCode.ERROR, None, False)
    monkeypatch.setattr(
        bootstrap_flow, "resolve_exit_behavior", lambda *a, **k: behavior
    )
    monkeypatch.setattr(bootstrap_flow, "_emit_fast_payload", lambda *_a, **_k: None)
    code = bootstrap_flow._emit_fast_error(
        "boom",
        error_type=ErrorType.INTERNAL,
        quiet=True,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
    )
    assert code == int(behavior.code)


def test_emit_fast_error_stream(monkeypatch: pytest.MonkeyPatch) -> None:
    from bijux_cli.core.enums import ExitCode
    from bijux_cli.core.exit_policy import ExitBehavior

    behavior = ExitBehavior(ExitCode.ERROR, "stdout", False)
    called: list[dict[str, object]] = []

    monkeypatch.setattr(
        bootstrap_flow, "resolve_exit_behavior", lambda *a, **k: behavior
    )
    monkeypatch.setattr(
        bootstrap_flow,
        "_emit_fast_payload",
        lambda payload, **_k: called.append(payload),
    )
    code = bootstrap_flow._emit_fast_error(
        "boom",
        error_type=ErrorType.INTERNAL,
        quiet=False,
        fmt=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
    )
    assert code == int(behavior.code)
    assert called == [{"error": "boom", "code": int(behavior.code)}]


def test_should_record_command_history_env_gate(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("BIJUXCLI_DISABLE_HISTORY", "1")
    assert bootstrap_flow.should_record_command_history(["status"]) is False


def test_should_record_command_history_skips_history_help() -> None:
    assert bootstrap_flow.should_record_command_history(["history"]) is False
    assert bootstrap_flow.should_record_command_history(["help"]) is False


def test_should_record_command_history_accepts_other() -> None:
    assert bootstrap_flow.should_record_command_history(["status"]) is True


def test_should_record_command_history_empty() -> None:
    assert bootstrap_flow.should_record_command_history([]) is False


def test_get_usage_for_args_basic() -> None:
    app = typer.Typer()

    @app.command()
    def status() -> None:
        return None

    text = bootstrap_flow.get_usage_for_args(["status"], app)
    assert "usage:" in text.lower()


def test_setup_structlog_uses_console_for_debug(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    seen: dict[str, object] = {}

    def fake_configure(*, processors: list[object], **_kwargs: object) -> None:
        seen["processors"] = processors

    monkeypatch.setattr(
        "bijux_cli.core.bootstrap_flow.structlog.configure", fake_configure
    )
    bootstrap_flow.setup_structlog(LogLevel.DEBUG)
    assert seen["processors"]


def test_setup_structlog_uses_console_for_test_mode(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    seen: dict[str, object] = {}

    def fake_configure(*, processors: list[object], **_kwargs: object) -> None:
        seen["processors"] = processors

    monkeypatch.setenv(ENV_TEST_MODE, "1")
    monkeypatch.setattr(
        "bijux_cli.core.bootstrap_flow.structlog.configure", fake_configure
    )
    bootstrap_flow.setup_structlog(None)
    assert seen["processors"]


def test_handle_version_flag(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr(
        "bijux_cli.core.bootstrap_flow.importlib_metadata.version",
        lambda _n: "1.2.3",
    )
    intent = _intent()
    assert bootstrap_flow._handle_version_request(["--version"], intent) == 0
    assert '"version": "1.2.3"' in capsys.readouterr().out


def test_handle_version_package_missing(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    import importlib.metadata as im

    monkeypatch.setattr(
        "bijux_cli.core.bootstrap_flow.importlib_metadata.version",
        lambda _n: (_ for _ in ()).throw(im.PackageNotFoundError()),
    )
    intent = _intent()
    assert bootstrap_flow._handle_version_request(["--version"], intent) == 0
    assert '"version": "unknown"' in capsys.readouterr().out


def test_handle_version_command_help(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    intent = _intent()
    monkeypatch.setattr(bootstrap_flow, "get_usage_for_args", lambda *_a, **_k: "help")
    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *a, **k: typer.Typer())
    assert bootstrap_flow._handle_version_request(["version", "--help"], intent) == 0
    assert "help" in capsys.readouterr().out


def test_handle_version_command_quiet(monkeypatch: pytest.MonkeyPatch) -> None:
    intent = replace(_intent(), quiet=True)
    assert bootstrap_flow._handle_version_request(["version"], intent) == 0


def test_handle_version_command_error(monkeypatch: pytest.MonkeyPatch) -> None:
    intent = _intent()
    monkeypatch.setattr(
        "bijux_cli.cli.commands.version._build_payload",
        lambda *_a: (_ for _ in ()).throw(ValueError("bad")),
    )
    monkeypatch.setattr(bootstrap_flow, "_emit_fast_error", lambda *_a, **_k: 2)
    assert bootstrap_flow._handle_version_request(["version"], intent) == 2


def test_handle_version_debug_message(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    intent = replace(_intent(), log_policy=resolve_log_policy(LogLevel.DEBUG))
    monkeypatch.setattr(
        "bijux_cli.cli.commands.version._build_payload", lambda *_a: {"version": "1"}
    )
    assert bootstrap_flow._handle_version_request(["version"], intent) == 0
    assert "debug: fast version path" in capsys.readouterr().err


def test_handle_help_request(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    intent = replace(_intent(), help=True)
    monkeypatch.setattr(bootstrap_flow, "get_usage_for_args", lambda *_a, **_k: "help")
    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *a, **k: typer.Typer())
    assert bootstrap_flow._handle_help_request(["--help"], intent) == 0
    assert "help" in capsys.readouterr().out


def test_handle_help_request_skips_when_disabled() -> None:
    intent = replace(_intent(), help=False)
    assert bootstrap_flow._handle_help_request(["status"], intent) is None


def test_run_runtime_success_records_history(monkeypatch: pytest.MonkeyPatch) -> None:
    history = _DummyHistory(entries=[])
    monkeypatch.setattr(
        bootstrap_flow.DIContainer, "current", lambda: _DummyContainer(history)
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_default_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_plugin_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(bootstrap_flow, "Engine", lambda *_a, **_k: None)
    monkeypatch.setattr(bootstrap_flow, "resolve_serializer", lambda: object())
    monkeypatch.setattr(bootstrap_flow, "resolve_emitter", lambda: object())
    monkeypatch.setattr(
        bootstrap_flow, "should_record_command_history", lambda *_a: True
    )

    class _App:
        def __call__(self, *args: object, **kwargs: object) -> int:
            return 0

    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *args, **kwargs: _App())

    exit_code = bootstrap_flow.run_runtime(_intent())
    assert exit_code == 0
    assert history.entries


def test_run_runtime_history_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    class _BadHistory:
        def add(self, **_kwargs: object) -> None:
            raise RuntimeError("fail")

    class _Container:
        def register(self, *_a: object, **_k: object) -> None:
            return None

        def resolve(self, service: object) -> object:
            if service is History:
                return _BadHistory()
            raise KeyError(service)

    monkeypatch.setattr(bootstrap_flow.DIContainer, "current", lambda: _Container())
    monkeypatch.setattr(
        bootstrap_flow, "register_default_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_plugin_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(bootstrap_flow, "Engine", lambda *_a, **_k: None)
    monkeypatch.setattr(bootstrap_flow, "resolve_serializer", lambda: object())
    monkeypatch.setattr(bootstrap_flow, "resolve_emitter", lambda: object())
    monkeypatch.setattr(
        bootstrap_flow, "should_record_command_history", lambda *_a: True
    )

    class _App:
        def __call__(self, *args: object, **kwargs: object) -> int:
            return 0

    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *args, **kwargs: _App())
    exit_code = bootstrap_flow.run_runtime(_intent())
    assert exit_code == 1


@pytest.mark.parametrize(
    "exc",
    [
        NoSuchOption("bad"),
        UsageError("usage"),
        UserInputError("input"),
        RuntimeError("boom"),
    ],
)
def test_run_runtime_errors_emit_payload(
    monkeypatch: pytest.MonkeyPatch, exc: Exception
) -> None:
    history = _DummyHistory(entries=[])
    monkeypatch.setattr(
        bootstrap_flow.DIContainer, "current", lambda: _DummyContainer(history)
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_default_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_plugin_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(bootstrap_flow, "Engine", lambda *_a, **_k: None)
    monkeypatch.setattr(bootstrap_flow, "resolve_serializer", lambda: object())
    monkeypatch.setattr(bootstrap_flow, "resolve_emitter", lambda: object())
    monkeypatch.setattr(
        bootstrap_flow, "should_record_command_history", lambda *_a: False
    )

    class _App:
        def __call__(self, *args: object, **kwargs: object) -> None:
            raise exc

    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *args, **kwargs: _App())

    captured: dict[str, Any] = {}

    def _emit(payload: object, **_kwargs: object) -> None:
        captured["payload"] = payload

    monkeypatch.setattr(bootstrap_flow, "emit_payload", _emit)

    exit_code = bootstrap_flow.run_runtime(_intent())
    assert exit_code in (1, 2)
    assert "payload" in captured


def test_run_runtime_keyboard_interrupt(monkeypatch: pytest.MonkeyPatch) -> None:
    history = _DummyHistory(entries=[])
    monkeypatch.setattr(
        bootstrap_flow.DIContainer, "current", lambda: _DummyContainer(history)
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_default_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_plugin_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(bootstrap_flow, "Engine", lambda *_a, **_k: None)

    class _Serializer:
        def dumps(self, *_args: object, **_kwargs: object) -> str:
            return "{}"

    monkeypatch.setattr(bootstrap_flow, "resolve_serializer", lambda: _Serializer())
    monkeypatch.setattr(bootstrap_flow, "resolve_emitter", lambda: object())
    monkeypatch.setattr(
        bootstrap_flow, "should_record_command_history", lambda *_a: False
    )

    class _App:
        def __call__(self, *args: object, **kwargs: object) -> None:
            raise KeyboardInterrupt

    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *args, **kwargs: _App())

    exit_code = bootstrap_flow.run_runtime(_intent())
    assert exit_code == 130


def test_run_runtime_handles_typer_exit(monkeypatch: pytest.MonkeyPatch) -> None:
    intent = _intent()

    class _App:
        def __call__(self, *args: object, **kwargs: object) -> int:
            raise typer.Exit(code=2)

    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *a, **k: _App())
    monkeypatch.setattr(bootstrap_flow, "resolve_serializer", lambda: object())
    monkeypatch.setattr(bootstrap_flow, "resolve_emitter", lambda: object())
    monkeypatch.setattr(
        bootstrap_flow, "register_default_services", lambda *a, **k: None
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_plugin_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(bootstrap_flow, "Engine", lambda: None)
    monkeypatch.setattr(
        bootstrap_flow, "should_record_command_history", lambda _a: False
    )
    assert bootstrap_flow.run_runtime(intent) == 2


def test_run_runtime_emit_error_stream_none(monkeypatch: pytest.MonkeyPatch) -> None:
    intent = _intent()
    from bijux_cli.core.enums import ExitCode
    from bijux_cli.core.exit_policy import ExitBehavior

    behavior = ExitBehavior(ExitCode.USAGE, None, False)
    monkeypatch.setattr(
        bootstrap_flow, "resolve_exit_behavior", lambda *a, **k: behavior
    )
    monkeypatch.setattr(
        bootstrap_flow,
        "emit_payload",
        lambda *_a, **_k: (_ for _ in ()).throw(AssertionError("emit should not run")),
    )

    class _App:
        def __call__(self, *args: object, **kwargs: object) -> None:
            raise UsageError("bad")

    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *a, **k: _App())
    monkeypatch.setattr(bootstrap_flow, "resolve_serializer", lambda: object())
    monkeypatch.setattr(bootstrap_flow, "resolve_emitter", lambda: object())
    monkeypatch.setattr(
        bootstrap_flow, "register_default_services", lambda *a, **k: None
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_plugin_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(bootstrap_flow, "Engine", lambda: None)
    monkeypatch.setattr(
        bootstrap_flow, "should_record_command_history", lambda _a: False
    )
    assert bootstrap_flow.run_runtime(intent) == int(behavior.code)


def test_run_runtime_quiet_suppresses_stderr(monkeypatch: pytest.MonkeyPatch) -> None:
    intent = _intent()
    intent = replace(intent, quiet=True, flags=replace(intent.flags, quiet=True))

    class _App:
        def __call__(self, *args: object, **kwargs: object) -> int:
            return 0

    monkeypatch.setattr(bootstrap_flow, "build_app", lambda *a, **k: _App())
    monkeypatch.setattr(bootstrap_flow, "resolve_serializer", lambda: object())
    monkeypatch.setattr(bootstrap_flow, "resolve_emitter", lambda: object())
    monkeypatch.setattr(
        bootstrap_flow, "register_default_services", lambda *a, **k: None
    )
    monkeypatch.setattr(
        bootstrap_flow, "register_plugin_services", lambda *_a, **_k: None
    )
    monkeypatch.setattr(bootstrap_flow, "Engine", lambda: None)
    monkeypatch.setattr(
        bootstrap_flow, "should_record_command_history", lambda _a: False
    )
    assert bootstrap_flow.run_runtime(intent) == 0


def test_main_returns_usage_on_intent_error(monkeypatch: pytest.MonkeyPatch) -> None:
    err = cast(
        FlagError,
        SimpleNamespace(message="bad", failure="invalid", flag="--bad"),
    )
    intent = replace(_intent(), errors=(err,))
    monkeypatch.setattr(bootstrap_flow, "build_cli_intent", lambda *_a, **_k: intent)
    monkeypatch.setattr(bootstrap_flow, "_emit_fast_error", lambda *_a, **_k: 2)
    assert bootstrap_flow.main() == 2


def test_main_startup_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    intent = _intent()
    monkeypatch.setattr(bootstrap_flow, "build_cli_intent", lambda *_a, **_k: intent)
    monkeypatch.setattr(
        bootstrap_flow,
        "setup_structlog",
        lambda *_a, **_k: (_ for _ in ()).throw(RuntimeError("boom")),
    )
    monkeypatch.setattr(bootstrap_flow, "_emit_fast_error", lambda *_a, **_k: 1)
    assert bootstrap_flow.main() == 1
