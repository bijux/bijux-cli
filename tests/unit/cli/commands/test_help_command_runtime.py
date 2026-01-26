# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit coverage for help command runtime helpers."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from bijux_cli.cli.commands import help_command as hc
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ColorMode, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.core.precedence import (
    EffectiveConfig,
    Flags,
    OutputConfig,
    resolve_log_policy,
)


def _configs() -> tuple[EffectiveConfig, OutputConfig]:
    flags = Flags(
        quiet=False,
        log_level=LogLevel.INFO,
        color=ColorMode.AUTO,
        format=OutputFormat.JSON,
    )
    effective = EffectiveConfig(flags=flags)
    output = OutputConfig(
        include_runtime=False,
        pretty=True,
        log_level=LogLevel.INFO,
        color=ColorMode.AUTO,
        format=OutputFormat.JSON,
        log_policy=resolve_log_policy(LogLevel.INFO),
    )
    return effective, output


def _call_help(*args: object, **kwargs: object) -> None:
    func = getattr(hc.help_callback, "__wrapped__", hc.help_callback)
    func(*args, **kwargs)


def test_resolve_help_config_from_container(monkeypatch: pytest.MonkeyPatch) -> None:
    effective, output = _configs()

    class _Container:
        def resolve(self, cls: object) -> object:
            if cls is EffectiveConfig:
                return effective
            if cls is OutputConfig:
                return output
            raise KeyError(cls)

    monkeypatch.setattr(DIContainer, "current", lambda: _Container())
    got_effective, got_output = hc._resolve_help_config()
    assert got_effective is effective
    assert got_output is output


def test_resolve_help_config_defaults(monkeypatch: pytest.MonkeyPatch) -> None:
    class _Container:
        def resolve(self, _cls: object) -> object:
            raise RuntimeError("missing")

    monkeypatch.setattr(DIContainer, "current", lambda: _Container())
    effective, output = hc._resolve_help_config()
    assert effective.flags.format is OutputFormat.JSON
    assert output.format is OutputFormat.JSON


def test_emit_structured_help_quiet() -> None:
    with pytest.raises(ExitIntentError) as excinfo:
        hc._emit_structured_help(
            command="help",
            payload={"help": "x"},
            output_format=OutputFormat.JSON,
            pretty=True,
            emit_output=False,
        )
    assert excinfo.value.intent.code == ExitCode.SUCCESS
    assert excinfo.value.intent.payload is None


def test_emit_structured_help_output() -> None:
    with pytest.raises(ExitIntentError) as excinfo:
        hc._emit_structured_help(
            command="help",
            payload={"help": "x"},
            output_format=OutputFormat.JSON,
            pretty=True,
            emit_output=True,
        )
    assert excinfo.value.intent.payload == {"help": "x"}


def test_emit_human_help_quiet() -> None:
    with pytest.raises(ExitIntentError) as excinfo:
        hc._emit_human_help(
            emit_output=False,
            color=False,
            help_text_provider=lambda: "help",
        )
    assert excinfo.value.intent.payload is None


def test_emit_human_help_output(monkeypatch: pytest.MonkeyPatch) -> None:
    emitted: list[str] = []
    monkeypatch.setattr(
        "bijux_cli.cli.commands.help_command.typer.echo",
        lambda msg, **_k: emitted.append(str(msg)),
    )
    with pytest.raises(ExitIntentError):
        hc._emit_human_help(
            emit_output=True,
            color=False,
            help_text_provider=lambda: "help",
        )
    assert emitted == ["help"]


def test_capture_help_text_prefers_return_value() -> None:
    assert hc._capture_help_text(lambda: "x") == "x"


def test_capture_help_text_uses_stdout() -> None:
    def _provider() -> str:
        print("printed")
        return ""

    assert hc._capture_help_text(_provider) == "printed\n"


def test_override_fmt_from_argv() -> None:
    argv = ["bijux", "help", "--format", "json"]
    monkeypatch = pytest.MonkeyPatch()
    monkeypatch.setattr("bijux_cli.cli.commands.help_command.sys.argv", argv)
    try:
        assert hc._override_fmt_from_argv("human") == "json"
    finally:
        monkeypatch.undo()


def test_help_callback_handles_help_flag(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(hc, "_resolve_help_config", lambda: _configs())
    monkeypatch.setattr(hc, "_find_target_command", lambda *_a, **_k: None)
    monkeypatch.setattr(
        "bijux_cli.cli.commands.help_command.sys.argv", ["bijux", "help", "--help"]
    )
    with pytest.raises(ExitIntentError):
        _call_help(SimpleNamespace(), command_path=None, fmt="human")
