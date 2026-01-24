# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the docs command."""

from __future__ import annotations

from pathlib import Path
import platform
from typing import Any, cast
from unittest.mock import MagicMock, patch

import pytest
from typer import Context
import yaml

import bijux_cli.cli.commands.diagnostics.docs as docs_pure
from bijux_cli.cli.commands.diagnostics.docs import (
    _build_spec_payload,
    _default_output_path,
    _resolve_output_target,
)
import bijux_cli.cli.commands.diagnostics.docs_command as docs_mod
from bijux_cli.cli.commands.diagnostics.docs_command import docs
from bijux_cli.core.enums import ColorMode, ErrorType, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.core.precedence import (
    EffectiveConfig,
    ExecutionPolicy,
    Flags,
    OutputConfig,
)


class FakeDocsService:
    def __init__(
        self,
        *,
        render_value: str = "",
        render_exc: Exception | None = None,
        write_exc: Exception | None = None,
    ) -> None:
        self._render_value = render_value
        self._render_exc = render_exc
        self._write_exc = write_exc

    def render(self, spec: dict[str, Any], *, fmt: Any, pretty: bool = False) -> str:
        _ = spec, fmt, pretty
        if self._render_exc:
            raise self._render_exc
        return self._render_value

    def write(
        self, spec: dict[str, Any], *, fmt: Any, name: str, pretty: bool = False
    ) -> str:
        _ = spec, fmt, pretty
        if self._write_exc:
            raise self._write_exc
        path = Path(name)
        path.write_text(self._render_value, encoding="utf-8")
        return str(path)


def call_docs(*args: Any, **kwargs: Any) -> Any:
    return docs.__wrapped__(*args, **kwargs)


def _fake_resolve_docs_config(
    **kwargs: Any,
) -> tuple[EffectiveConfig, OutputConfig]:
    fmt = (kwargs.get("fmt") or "json").lower()
    output_format = OutputFormat.YAML if fmt == "yaml" else OutputFormat.JSON
    raw_level = kwargs.get("log_level", LogLevel.INFO)
    log_level = (
        raw_level
        if isinstance(raw_level, LogLevel)
        else LogLevel(str(raw_level).lower())
    )
    pretty = bool(kwargs.get("pretty", False))
    policy = ExecutionPolicy(
        output_format=output_format,
        color=ColorMode.AUTO,
        quiet=bool(kwargs.get("quiet", False)),
        log_level=log_level,
        pretty=pretty,
        include_runtime=bool(kwargs.get("include_runtime", False)),
    )
    effective = EffectiveConfig(
        flags=Flags(
            quiet=policy.quiet,
            log_level=policy.log_level,
            color=policy.color,
            format=policy.output_format,
        )
    )
    output = OutputConfig(
        include_runtime=policy.include_runtime,
        pretty=policy.pretty,
        log_level=policy.log_level,
        color=policy.color,
        format=policy.output_format,
        log_policy=policy.log_policy,
    )
    return effective, output


def test_default_output_path() -> None:
    """Test the default output path generation for different formats."""
    base = Path("/some/base")
    assert _default_output_path(base, OutputFormat.JSON) == base / "spec.json"
    assert _default_output_path(base, OutputFormat.YAML) == base / "spec.yaml"


def test_resolve_output_target(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the resolution of the output target path."""
    monkeypatch.setattr(Path, "cwd", classmethod(lambda cls: Path("/cwd")))
    tgt, p = _resolve_output_target(None, OutputFormat.JSON)
    assert tgt == "/cwd/spec.json"
    assert isinstance(p, Path)
    assert p.name == "spec.json"

    out = Path("-")
    tgt, p = _resolve_output_target(out, OutputFormat.YAML)
    assert tgt == "-"
    assert p is None

    d = tmp_path / "outdir"
    d.mkdir()
    tgt, p = _resolve_output_target(d, OutputFormat.JSON)
    assert tgt == str(d / "spec.json")
    assert p == d / "spec.json"

    f = tmp_path / "foo.bar"
    tgt, p = _resolve_output_target(f, OutputFormat.YAML)
    assert tgt == str(f)
    assert p == f


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.raise_exit_intent", autospec=True
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
def test_docs_stray_args_option(mock_resolve: MagicMock, mock_emit: MagicMock) -> None:
    """Test that a stray option causes a structured error and exit."""
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_docs_config
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = ["-x"]
    with pytest.raises(SystemExit):
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=False,
            log_level=LogLevel.INFO,
        )
    mock_emit.assert_called_once_with(
        "No such option: -x",
        code=2,
        failure="args",
        error_type=ErrorType.USAGE,
        command="docs",
        fmt=OutputFormat.JSON,
        quiet=False,
        include_runtime=False,
        log_level=LogLevel.INFO,
    )


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.raise_exit_intent", autospec=True
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
def test_docs_stray_args_word(mock_resolve: MagicMock, mock_emit: MagicMock) -> None:
    """Test that a stray argument causes a structured error and exit."""
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_docs_config
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = ["foo"]
    with pytest.raises(SystemExit):
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=False,
            log_level=LogLevel.INFO,
        )
    mock_emit.assert_called_once_with(
        "Too many arguments: foo",
        code=2,
        failure="args",
        error_type=ErrorType.USAGE,
        command="docs",
        fmt=OutputFormat.JSON,
        quiet=False,
        include_runtime=False,
        log_level=LogLevel.INFO,
    )


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.contains_non_ascii_env",
    autospec=True,
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.raise_exit_intent", autospec=True
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
def test_docs_ascii_env_failure(
    mock_resolve: MagicMock, mock_emit: MagicMock, mock_nonascii: MagicMock
) -> None:
    """Test that non-ASCII environment variables cause an error and exit."""
    mock_nonascii.return_value = True
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_docs_config
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(SystemExit):
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=False,
            log_level=LogLevel.INFO,
        )
    mock_emit.assert_called_once_with(
        "Non-ASCII characters in environment variables",
        code=3,
        failure="ascii_env",
        error_type=ErrorType.ASCII,
        command="docs",
        fmt=OutputFormat.JSON,
        quiet=False,
        include_runtime=False,
        log_level=LogLevel.INFO,
    )


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._build_spec_payload", autospec=True
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.raise_exit_intent", autospec=True
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.contains_non_ascii_env",
    autospec=True,
)
def test_docs_ascii_payload_failure(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    mock_emit: MagicMock,
    mock_build: MagicMock,
) -> None:
    """Test that a payload builder error causes a structured error and exit."""
    mock_nonascii.return_value = False
    mock_build.side_effect = ValueError("bad payload")
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_docs_config
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(SystemExit):
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=False,
            log_level=LogLevel.INFO,
        )
    mock_emit.assert_called_once_with(
        "bad payload",
        code=3,
        failure="ascii",
        error_type=ErrorType.ASCII,
        command="docs",
        fmt=OutputFormat.JSON,
        quiet=False,
        include_runtime=False,
        log_level=LogLevel.INFO,
    )


def test_default_output_and_resolve_targets(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test resolution of various output targets."""
    base = tmp_path / "base"
    base.mkdir()
    assert _default_output_path(base, OutputFormat.JSON) == base / "spec.json"
    assert _default_output_path(base, OutputFormat.YAML) == base / "spec.yaml"

    monkeypatch.chdir(tmp_path)
    tgt, path = _resolve_output_target(None, OutputFormat.JSON)
    assert tgt.endswith("spec.json")
    assert isinstance(path, Path)

    tgt, path = _resolve_output_target(Path("-"), OutputFormat.YAML)
    assert tgt == "-"
    assert path is None

    d = tmp_path / "d"
    d.mkdir()
    tgt, path = _resolve_output_target(d, OutputFormat.YAML)
    assert tgt.endswith("spec.yaml")
    assert path == d / "spec.yaml"

    f = tmp_path / "foo.out"
    tgt, path = _resolve_output_target(f, OutputFormat.JSON)
    assert tgt == str(f)
    assert path == f


def test_build_spec_payload_basic(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the basic structure of the specification payload."""
    monkeypatch.setattr(docs_pure, "CLI_VERSION", "vX.Y.Z")
    import bijux_cli.cli.commands as cmd_pkg

    monkeypatch.setattr(
        cmd_pkg, "list_registered_command_names", lambda: ["one", "two"]
    )

    payload = _build_spec_payload(include_runtime=False)
    assert payload["version"] == "vX.Y.Z"
    assert payload["commands"] == ["one", "two"]
    assert payload.get("python") is None
    assert payload.get("platform") is None

    payload_rt = _build_spec_payload(include_runtime=True)
    assert payload_rt["python"] == platform.python_version()
    assert payload_rt["platform"] == platform.platform()


def test_build_spec_payload_ascii_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an ASCII safety check failure raises an error."""
    monkeypatch.setenv("DUMMY", "")
    monkeypatch.setattr(
        docs_pure,
        "ascii_safe",
        lambda v, k: (_ for _ in ()).throw(ValueError("bad ascii")),
    )
    import bijux_cli.cli.commands as cmd_pkg

    monkeypatch.setattr(cmd_pkg, "list_registered_command_names", lambda: [])
    with pytest.raises(ValueError, match=r"bad ascii"):
        _build_spec_payload(include_runtime=False)


def test_docs_stdout_branch(
    capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that specifying stdout ('-') prints the spec to stdout."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", "")
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(docs_mod, "_resolve_docs_config", _fake_resolve_docs_config)
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"val": 3})
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="JSON_OUT"),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(ExitIntentError) as ei:
        call_docs(
            ctx,
            out=Path("-"),
            quiet=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

    assert ei.value.intent.code == 0
    assert capsys.readouterr().out == "JSON_OUT\n"


def test_docs_file_written_and_exit_intent(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the spec is written to a file and a success payload is emitted."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", str(tmp_path))
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(docs_mod, "_resolve_docs_config", _fake_resolve_docs_config)
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"hello": "world"})

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value='{"hello":"world"}'),
    )
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_config",
        lambda **_kw: _fake_resolve_docs_config(
            fmt="json",
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=False,
            include_runtime=True,
        ),
    )

    monkeypatch.delenv("BIJUXCLI_TEST_IO_FAIL", raising=False)

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(ExitIntentError) as exc:
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=False,
            log_level=LogLevel.INFO,
        )

    spec_file = tmp_path / "spec.json"
    assert spec_file.read_text(encoding="utf-8") == '{"hello":"world"}'
    intent = exc.value.intent
    assert intent.payload == {"status": "written", "file": str(spec_file)}


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.contains_non_ascii_env",
    autospec=True,
)
def test_docs_write_failure(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a file write failure is handled gracefully."""
    mock_nonascii.return_value = False
    mock_resolve.side_effect = _fake_resolve_docs_config

    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", str(tmp_path))
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"b": 2})
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="{}"),
    )

    def broken_write_text(self: Path, content: str, encoding: str) -> None:
        raise OSError("disk full")

    monkeypatch.setattr(Path, "write_text", broken_write_text)

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(ExitIntentError) as exc:
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

    intent = exc.value.intent
    payload = cast(dict[str, Any], intent.payload)
    assert payload["failure"] == "write"
    assert "disk full" in payload["error"]


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.contains_non_ascii_env",
    autospec=True,
)
def test_docs_missing_output_dir(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a non-existent output directory causes an error."""
    mock_nonascii.return_value = False
    mock_resolve.side_effect = _fake_resolve_docs_config

    bad_dir = tmp_path / "no" / "such" / "dir"
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", str(bad_dir))

    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"a": 1})
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="{}"),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(ExitIntentError) as exc:
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

    intent = exc.value.intent
    payload = cast(dict[str, Any], intent.payload)
    assert payload["failure"] == "output_dir"
    assert str(bad_dir.parent) in payload["error"]


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.contains_non_ascii_env",
    autospec=True,
)
def test_docs_writes_yaml_and_emit(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that YAML output is correctly serialized and written."""
    mock_nonascii.return_value = False
    mock_resolve.side_effect = lambda **_kw: _fake_resolve_docs_config(
        fmt="yaml",
        quiet=False,
        log_level=LogLevel.INFO,
        pretty=False,
        include_runtime=False,
    )
    monkeypatch.delenv("BIJUXCLI_TEST_IO_FAIL", raising=False)
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", str(tmp_path))
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"foo": "bar"})
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(
            render_value=yaml.safe_dump({"foo": "bar"}, sort_keys=False)
        ),
    )
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(ExitIntentError) as exc:
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="yaml",
            pretty=False,
            log_level=LogLevel.INFO,
        )
    spec_file = tmp_path / "spec.yaml"
    text = spec_file.read_text(encoding="utf-8")
    assert "foo: bar" in text
    payload = cast(dict[str, Any], exc.value.intent.payload)
    assert payload["status"] == "written"


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.raise_exit_intent", autospec=True
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.contains_non_ascii_env",
    autospec=True,
)
def test_docs_io_fail_flag(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    mock_emit: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a simulated I/O failure flag is handled."""
    mock_nonascii.return_value = False
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_docs_config

    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", str(tmp_path))
    monkeypatch.setenv("BIJUXCLI_TEST_IO_FAIL", "1")

    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"x": 42})
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="{}"),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(SystemExit):
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

    mock_emit.assert_called_once_with(
        "Simulated I/O failure for test",
        code=1,
        failure="io_fail",
        error_type=ErrorType.INTERNAL,
        command="docs",
        fmt=OutputFormat.JSON,
        quiet=False,
        include_runtime=False,
        log_level=LogLevel.INFO,
    )


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.raise_exit_intent", autospec=True
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.contains_non_ascii_env",
    autospec=True,
)
def test_docs_internal_error_path_none(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    mock_emit: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test handling of an internal error where the resolved output path is None."""
    mock_nonascii.return_value = False
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_docs_config

    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"k": "v"})
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="{}"),
    )

    monkeypatch.setattr(
        docs_mod, "_resolve_output_target", lambda out, fmt: ("weird", None)
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(SystemExit):
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

    mock_emit.assert_called_once_with(
        "Internal error: expected non-null output path",
        code=1,
        failure="internal",
        error_type=ErrorType.INTERNAL,
        command="docs",
        fmt=OutputFormat.JSON,
        quiet=False,
        include_runtime=False,
        log_level=LogLevel.INFO,
    )


def test_docs_stdout_debug_no_diagnostics(
    capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that debug mode with stdout output does not print diagnostics."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", "")
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(docs_mod, "_resolve_docs_config", _fake_resolve_docs_config)
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"num": 7})
    monkeypatch.setattr(docs_mod, "record_history", lambda *_a, **_k: None)

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="DUMP"),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(ExitIntentError) as ei:
        call_docs(
            ctx,
            out=Path("-"),
            quiet=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.DEBUG,
        )

    out, err = capsys.readouterr()
    assert out == "DUMP\n"
    assert err == ""
    assert ei.value.intent.code == 0


def test_docs_stdout_quiet_skips_echo(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that quiet mode with stdout output produces no output."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", "")
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(docs_mod, "_resolve_docs_config", _fake_resolve_docs_config)
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"a": 1})
    monkeypatch.setattr(docs_mod, "record_history", lambda *_a, **_k: None)

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="X"),
    )
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_config",
        lambda **_kw: _fake_resolve_docs_config(
            fmt="json",
            quiet=True,
            log_level=LogLevel.ERROR,
            pretty=True,
            include_runtime=False,
        ),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(ExitIntentError) as exc:
        call_docs(
            ctx,
            out=Path("-"),
            quiet=True,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

    assert exc.value.intent.code == 0
    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


def test_docs_stdout_yaml(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that YAML output is correctly echoed to stdout."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", "")
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(docs_mod, "_resolve_docs_config", _fake_resolve_docs_config)
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"hello": "world"})
    monkeypatch.setattr(docs_mod, "record_history", lambda *_a, **_k: None)

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="{hello: world}\n"),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(ExitIntentError) as exc:
        call_docs(
            ctx,
            out=Path("-"),
            quiet=False,
            fmt="yaml",
            pretty=False,
            log_level=LogLevel.INFO,
        )

    assert exc.value.intent.code == 0
    out, err = capsys.readouterr()
    assert out.strip() == "{hello: world}"
    assert err == ""


@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.raise_exit_intent", autospec=True
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command._resolve_docs_config",
    autospec=True,
)
@patch(
    "bijux_cli.cli.commands.diagnostics.docs_command.contains_non_ascii_env",
    autospec=True,
)
def test_docs_yaml_serialization_failure(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    mock_emit: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a YAML serialization failure is handled gracefully."""
    mock_nonascii.return_value = False
    mock_resolve.side_effect = lambda **_kw: _fake_resolve_docs_config(
        fmt="yaml",
        quiet=False,
        log_level=LogLevel.INFO,
        pretty=True,
        include_runtime=False,
    )
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", str(tmp_path))
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"foo": "bar"})

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_exc=RuntimeError("yaml‐oops")),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    mock_emit.side_effect = SystemExit()

    with pytest.raises(SystemExit):
        call_docs(
            ctx,
            out=None,
            quiet=False,
            fmt="yaml",
            pretty=True,
            log_level=LogLevel.INFO,
        )

    mock_emit.assert_called_once_with(
        "Serialization failed: yaml‐oops",
        code=1,
        failure="serialize",
        error_type=ErrorType.INTERNAL,
        command="docs",
        fmt=OutputFormat.YAML,
        quiet=False,
        include_runtime=False,
        log_level=LogLevel.INFO,
    )
