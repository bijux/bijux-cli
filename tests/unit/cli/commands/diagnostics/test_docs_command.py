# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for docs command runtime behavior."""

from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace

import pytest

from bijux_cli.cli.commands.diagnostics import docs_command as dc
from bijux_cli.cli.core.constants import ENV_DOCS_OUT, ENV_TEST_IO_FAIL
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ColorMode, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.core.precedence import (
    EffectiveConfig,
    Flags,
    OutputConfig,
    resolve_log_policy,
)


def _configs(*, quiet: bool = False) -> tuple[EffectiveConfig, OutputConfig]:
    flags = Flags(
        quiet=quiet,
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


def _ctx(args: list[str] | None = None) -> SimpleNamespace:
    return SimpleNamespace(args=args or [])


def _call_docs(*args: object, **kwargs: object) -> None:
    kwargs.setdefault("fmt", "json")
    kwargs.setdefault("quiet", False)
    kwargs.setdefault("pretty", True)
    kwargs.setdefault("log_level", "info")
    func = getattr(dc.docs, "__wrapped__", dc.docs)
    func(*args, **kwargs)


class _DocsService:
    def __init__(self) -> None:
        self.written: list[tuple[dict[str, object], str, str, bool]] = []

    def render(self, _spec: dict[str, object], *, fmt: str, pretty: bool) -> str:
        return f"{fmt}:{pretty}"

    def write(
        self, spec: dict[str, object], *, fmt: str, name: str, pretty: bool
    ) -> None:
        self.written.append((spec, fmt, name, pretty))


def test_resolve_docs_config_from_container(monkeypatch: pytest.MonkeyPatch) -> None:
    effective, output = _configs()

    class _Container:
        def resolve(self, cls: object) -> object:
            if cls is EffectiveConfig:
                return effective
            if cls is OutputConfig:
                return output
            raise KeyError(cls)

    monkeypatch.setattr(DIContainer, "current", lambda: _Container())
    resolved_effective, resolved_output = dc._resolve_docs_config()
    assert resolved_effective is effective
    assert resolved_output is output


def test_resolve_docs_config_defaults(monkeypatch: pytest.MonkeyPatch) -> None:
    class _Container:
        def resolve(self, _cls: object) -> object:
            raise RuntimeError("missing")

    monkeypatch.setattr(DIContainer, "current", lambda: _Container())
    effective, output = dc._resolve_docs_config()
    assert effective.flags.format is OutputFormat.JSON
    assert output.format is OutputFormat.JSON


def test_docs_rejects_non_ascii_env(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "contains_non_ascii_env", lambda: True)
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx([]))


def test_docs_rejects_stray_args(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx(["--oops"]))


def test_docs_env_out_overrides_out(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setenv(ENV_DOCS_OUT, str(tmp_path / "out.json"))
    seen: dict[str, Path] = {}

    def fake_resolve(out: Path | None, fmt: str) -> tuple[str, Path | None]:
        assert out is not None
        seen["path"] = out
        return "-", None

    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "_resolve_output_target", fake_resolve)
    monkeypatch.setattr(dc, "_build_spec_payload", lambda _r: {"v": 1})
    monkeypatch.setattr(dc, "_spec_mapping", lambda spec: spec)
    monkeypatch.setattr(dc, "_resolve_docs_service", lambda: _DocsService())
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx([]), out=None)
    assert seen["path"].name == "out.json"


def test_docs_build_spec_value_error(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(
        dc, "_build_spec_payload", lambda _r: (_ for _ in ()).throw(ValueError("bad"))
    )
    monkeypatch.setattr(dc, "_resolve_output_target", lambda _o, _f: ("-", None))
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx([]))


def test_docs_render_error(monkeypatch: pytest.MonkeyPatch) -> None:
    class _FailingService(_DocsService):
        def render(self, _spec: dict[str, object], *, fmt: str, pretty: bool) -> str:
            raise RuntimeError("boom")

    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "_resolve_output_target", lambda _o, _f: ("-", None))
    monkeypatch.setattr(dc, "_build_spec_payload", lambda _r: {"v": 1})
    monkeypatch.setattr(dc, "_spec_mapping", lambda spec: spec)
    monkeypatch.setattr(dc, "_resolve_docs_service", lambda: _FailingService())
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx([]))


def test_docs_env_io_fail(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "_resolve_output_target", lambda _o, _f: ("-", None))
    monkeypatch.setattr(dc, "_build_spec_payload", lambda _r: {"v": 1})
    monkeypatch.setattr(dc, "_spec_mapping", lambda spec: spec)
    monkeypatch.setattr(dc, "_resolve_docs_service", lambda: _DocsService())
    monkeypatch.setenv(ENV_TEST_IO_FAIL, "1")
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx([]))


def test_docs_stdout_target_emits(monkeypatch: pytest.MonkeyPatch) -> None:
    service = _DocsService()
    emitted: list[str] = []
    recorded: list[tuple[str, int]] = []

    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "_resolve_output_target", lambda _o, _f: ("-", None))
    monkeypatch.setattr(dc, "_build_spec_payload", lambda _r: {"v": 1})
    monkeypatch.setattr(dc, "_spec_mapping", lambda spec: spec)
    monkeypatch.setattr(dc, "_resolve_docs_service", lambda: service)
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.docs_command.typer.echo",
        lambda msg, **_k: emitted.append(str(msg)),
    )
    monkeypatch.setattr(
        dc, "record_history", lambda cmd, code: recorded.append((cmd, code))
    )

    with pytest.raises(ExitIntentError) as excinfo:
        _call_docs(_ctx([]))
    assert emitted
    assert recorded == [("docs", 0)]
    assert excinfo.value.intent.code == ExitCode.SUCCESS


def test_docs_missing_output_path(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "_resolve_output_target", lambda _o, _f: ("file", None))
    monkeypatch.setattr(dc, "_build_spec_payload", lambda _r: {"v": 1})
    monkeypatch.setattr(dc, "_spec_mapping", lambda spec: spec)
    monkeypatch.setattr(dc, "_resolve_docs_service", lambda: _DocsService())
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx([]), out=Path("out.json"))


def test_docs_missing_output_dir(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    out_path = tmp_path / "missing" / "out.json"
    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "_resolve_output_target", lambda _o, _f: ("file", out_path))
    monkeypatch.setattr(dc, "_build_spec_payload", lambda _r: {"v": 1})
    monkeypatch.setattr(dc, "_spec_mapping", lambda spec: spec)
    monkeypatch.setattr(dc, "_resolve_docs_service", lambda: _DocsService())
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx([]), out=out_path)


def test_docs_write_error(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    out_path = tmp_path / "out.json"

    class _FailingService(_DocsService):
        def write(self, *_a: object, **_k: object) -> None:
            raise RuntimeError("disk full")

    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "_resolve_output_target", lambda _o, _f: ("file", out_path))
    monkeypatch.setattr(dc, "_build_spec_payload", lambda _r: {"v": 1})
    monkeypatch.setattr(dc, "_spec_mapping", lambda spec: spec)
    monkeypatch.setattr(dc, "_resolve_docs_service", lambda: _FailingService())
    with pytest.raises(ExitIntentError):
        _call_docs(_ctx([]), out=out_path)


def test_docs_write_success(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    out_path = tmp_path / "out.json"
    service = _DocsService()
    recorded: list[tuple[str, int]] = []

    monkeypatch.setattr(dc, "_resolve_docs_config", lambda: _configs())
    monkeypatch.setattr(dc, "_resolve_output_target", lambda _o, _f: ("file", out_path))
    monkeypatch.setattr(dc, "_build_spec_payload", lambda _r: {"v": 1})
    monkeypatch.setattr(dc, "_spec_mapping", lambda spec: spec)
    monkeypatch.setattr(dc, "_resolve_docs_service", lambda: service)
    monkeypatch.setattr(
        dc, "record_history", lambda cmd, code: recorded.append((cmd, code))
    )

    with pytest.raises(ExitIntentError) as excinfo:
        _call_docs(_ctx([]), out=out_path)
    assert service.written
    assert recorded == [("docs", 0)]
    assert excinfo.value.intent.payload == {"status": "written", "file": str(out_path)}
