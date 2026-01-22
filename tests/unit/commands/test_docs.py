# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the docs command."""

from __future__ import annotations

from pathlib import Path
import platform
from typing import Any
from unittest.mock import MagicMock, patch

import pytest
from typer import Context, Exit
import yaml

import bijux_cli.cli.commands.diagnostics.docs as docs_mod
from bijux_cli.cli.commands.diagnostics.docs import (
    _build_spec_payload,
    _default_output_path,
    _resolve_output_target,
    docs,
)
from bijux_cli.core.enums import ColorMode, OutputFormat
from bijux_cli.core.precedence import ExecutionPolicy


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


def _fake_resolve_command_config(
    **kwargs: Any,
) -> tuple[ExecutionPolicy, OutputFormat, str]:
    fmt = (kwargs.get("fmt") or "json").lower()
    output_format = OutputFormat.YAML if fmt == "yaml" else OutputFormat.JSON
    verbose = bool(kwargs.get("verbose", False))
    log_level = str(kwargs.get("log_level", "info")).lower()
    pretty = bool(kwargs.get("pretty", False))
    return (
        ExecutionPolicy(
            output_format=fmt,
            color=ColorMode.AUTO,
            quiet=bool(kwargs.get("quiet", False)),
            verbose=verbose,
            verbose_level=1 if verbose else 0,
            log_level=log_level,
            pretty=pretty,
            include_runtime=verbose,
            json=False,
        ),
        output_format,
        fmt,
    )


def test_default_output_path() -> None:
    """Test the default output path generation for different formats."""
    base = Path("/some/base")
    assert _default_output_path(base, "json") == base / "spec.json"
    assert _default_output_path(base, "yaml") == base / "spec.yaml"


def test_resolve_output_target(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the resolution of the output target path."""
    monkeypatch.setattr(Path, "cwd", classmethod(lambda cls: Path("/cwd")))
    tgt, p = _resolve_output_target(None, "json")
    assert tgt == "/cwd/spec.json"
    assert isinstance(p, Path)
    assert p.name == "spec.json"

    out = Path("-")
    tgt, p = _resolve_output_target(out, "yaml")
    assert tgt == "-"
    assert p is None

    d = tmp_path / "outdir"
    d.mkdir()
    tgt, p = _resolve_output_target(d, "json")
    assert tgt == str(d / "spec.json")
    assert p == d / "spec.json"

    f = tmp_path / "foo.bar"
    tgt, p = _resolve_output_target(f, "yaml")
    assert tgt == str(f)
    assert p == f


@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
def test_docs_stray_args_option(mock_resolve: MagicMock, mock_emit: MagicMock) -> None:
    """Test that a stray option causes a structured error and exit."""
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_command_config
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = ["-x"]
    with pytest.raises(SystemExit):
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            log_level="info",
        )
    mock_emit.assert_called_once_with(
        "No such option: -x",
        code=2,
        failure="args",
        command="docs",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
def test_docs_stray_args_word(mock_resolve: MagicMock, mock_emit: MagicMock) -> None:
    """Test that a stray argument causes a structured error and exit."""
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_command_config
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = ["foo"]
    with pytest.raises(SystemExit):
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            log_level="info",
        )
    mock_emit.assert_called_once_with(
        "Too many arguments: foo",
        code=2,
        failure="args",
        command="docs",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


@patch("bijux_cli.cli.commands.diagnostics.docs.contains_non_ascii_env", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
def test_docs_ascii_env_failure(
    mock_resolve: MagicMock, mock_emit: MagicMock, mock_nonascii: MagicMock
) -> None:
    """Test that non-ASCII environment variables cause an error and exit."""
    mock_nonascii.return_value = True
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_command_config
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(SystemExit):
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            log_level="info",
        )
    mock_emit.assert_called_once_with(
        "Non-ASCII characters in environment variables",
        code=3,
        failure="ascii_env",
        command="docs",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


@patch("bijux_cli.cli.commands.diagnostics.docs._build_spec_payload", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.contains_non_ascii_env", autospec=True)
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
    mock_resolve.side_effect = _fake_resolve_command_config
    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(SystemExit):
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            log_level="info",
        )
    mock_emit.assert_called_once_with(
        "bad payload",
        code=3,
        failure="ascii",
        command="docs",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


def test_default_output_and_resolve_targets(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test resolution of various output targets."""
    base = tmp_path / "base"
    base.mkdir()
    assert _default_output_path(base, "json") == base / "spec.json"
    assert _default_output_path(base, "yaml") == base / "spec.yaml"

    monkeypatch.chdir(tmp_path)
    tgt, path = _resolve_output_target(None, "json")
    assert tgt.endswith("spec.json")
    assert isinstance(path, Path)

    tgt, path = _resolve_output_target(Path("-"), "yaml")
    assert tgt == "-"
    assert path is None

    d = tmp_path / "d"
    d.mkdir()
    tgt, path = _resolve_output_target(d, "yaml")
    assert tgt.endswith("spec.yaml")
    assert path == d / "spec.yaml"

    f = tmp_path / "foo.out"
    tgt, path = _resolve_output_target(f, "json")
    assert tgt == str(f)
    assert path == f


def test_build_spec_payload_basic(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the basic structure of the specification payload."""
    monkeypatch.setattr(docs_mod, "CLI_VERSION", "vX.Y.Z")
    import bijux_cli.cli.commands as cmd_pkg

    monkeypatch.setattr(
        cmd_pkg, "list_registered_command_names", lambda: ["one", "two"]
    )

    payload = _build_spec_payload(include_runtime=False)
    assert payload["version"] == "vX.Y.Z"
    assert payload["commands"] == ["one", "two"]
    assert "python" not in payload
    assert "platform" not in payload

    payload_rt = _build_spec_payload(include_runtime=True)
    assert payload_rt["python"] == platform.python_version()
    assert payload_rt["platform"] == platform.platform()


def test_build_spec_payload_ascii_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an ASCII safety check failure raises an error."""
    monkeypatch.setenv("DUMMY", "")
    monkeypatch.setattr(
        "bijux_cli.cli.validation.ascii_safe",
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
    monkeypatch.setattr(
        docs_mod, "resolve_command_config", _fake_resolve_command_config
    )
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"val": 3})
    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="JSON_OUT"),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(Exit) as ei:
        docs(
            ctx,
            out=Path("-"),
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )

    assert ei.value.exit_code == 0
    assert capsys.readouterr().out == "JSON_OUT\n"


def test_docs_file_written_and_emit_and_exit(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the spec is written to a file and a success payload is emitted."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", str(tmp_path))
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(
        docs_mod, "resolve_command_config", _fake_resolve_command_config
    )
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"hello": "world"})

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value='{"hello":"world"}'),
    )
    monkeypatch.setattr(
        docs_mod,
        "resolve_command_config",
        lambda **_kw: (
            ExecutionPolicy(
                output_format="json",
                color=ColorMode.AUTO,
                quiet=False,
                verbose=True,
                verbose_level=1,
                log_level="info",
                pretty=False,
                include_runtime=True,
                json=False,
            ),
            OutputFormat.JSON,
            "json",
        ),
    )

    monkeypatch.delenv("BIJUXCLI_TEST_IO_FAIL", raising=False)

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with patch("bijux_cli.cli.commands.diagnostics.docs.emit_and_exit") as mock_emit:
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=True,
            fmt="json",
            pretty=False,
            log_level="info",
        )

    spec_file = tmp_path / "spec.json"
    assert spec_file.read_text(encoding="utf-8") == '{"hello":"world"}'
    mock_emit.assert_called_once_with(
        {"status": "written", "file": str(spec_file)},
        OutputFormat.JSON,
        False,
        True,
        False,
        False,
        "docs",
    )


@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.contains_non_ascii_env", autospec=True)
def test_docs_write_failure(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    mock_emit: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a file write failure is handled gracefully."""
    mock_nonascii.return_value = False
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_command_config

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
    with pytest.raises(SystemExit):
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )

    mock_emit.assert_called_once_with(
        "Failed to write spec: disk full",
        code=2,
        failure="write",
        command="docs",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.contains_non_ascii_env", autospec=True)
def test_docs_missing_output_dir(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    mock_emit: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a non-existent output directory causes an error."""
    mock_nonascii.return_value = False
    mock_emit.side_effect = SystemExit()
    mock_resolve.side_effect = _fake_resolve_command_config

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
    with pytest.raises(SystemExit):
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )

    bad_parent = bad_dir.parent
    mock_emit.assert_called_once_with(
        f"Output directory does not exist: {bad_parent}",
        code=2,
        failure="output_dir",
        command="docs",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


@patch("bijux_cli.cli.commands.diagnostics.docs.emit_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.contains_non_ascii_env", autospec=True)
def test_docs_writes_yaml_and_emit(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    mock_emit: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that YAML output is correctly serialized and written."""
    mock_nonascii.return_value = False
    mock_resolve.side_effect = lambda **_kw: (
        ExecutionPolicy(
            output_format="yaml",
            color=ColorMode.AUTO,
            quiet=False,
            verbose=False,
            verbose_level=0,
            log_level="info",
            pretty=False,
            include_runtime=False,
            json=False,
        ),
        OutputFormat.YAML,
        "yaml",
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
    docs(
        ctx,
        out=None,
        quiet=False,
        verbose=False,
        fmt="yaml",
        pretty=False,
        log_level="info",
    )
    spec_file = tmp_path / "spec.yaml"
    text = spec_file.read_text(encoding="utf-8")
    assert "foo: bar" in text
    mock_emit.assert_called_once_with(
        {"status": "written", "file": str(spec_file)},
        OutputFormat.YAML,
        False,
        False,
        False,
        False,
        "docs",
    )


@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.contains_non_ascii_env", autospec=True)
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
    mock_resolve.side_effect = _fake_resolve_command_config

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
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )

    mock_emit.assert_called_once_with(
        "Simulated I/O failure for test",
        code=1,
        failure="io_fail",
        command="docs",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.contains_non_ascii_env", autospec=True)
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
    mock_resolve.side_effect = _fake_resolve_command_config

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
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )

    mock_emit.assert_called_once_with(
        "Internal error: expected non-null output path",
        code=1,
        failure="internal",
        command="docs",
        fmt="json",
        quiet=False,
        include_runtime=False,
        debug=False,
    )


def test_docs_stdout_debug_no_diagnostics(
    capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that debug mode with stdout output does not print diagnostics."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", "")
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(
        docs_mod, "resolve_command_config", _fake_resolve_command_config
    )
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"num": 7})

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="DUMP"),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(Exit) as ei:
        docs(
            ctx,
            out=Path("-"),
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="debug",
        )

    out, err = capsys.readouterr()
    assert out == "DUMP\n"
    assert err == ""
    assert ei.value.exit_code == 0


def test_docs_stdout_quiet_skips_echo(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that quiet mode with stdout output produces no output."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", "")
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(
        docs_mod, "resolve_command_config", _fake_resolve_command_config
    )
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"a": 1})

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="X"),
    )
    monkeypatch.setattr(
        docs_mod,
        "resolve_command_config",
        lambda **_kw: (
            ExecutionPolicy(
                output_format="json",
                color=ColorMode.AUTO,
                quiet=True,
                verbose=False,
                verbose_level=0,
                log_level="error",
                pretty=True,
                include_runtime=False,
                json=False,
            ),
            OutputFormat.JSON,
            "json",
        ),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(Exit) as exc:
        docs(
            ctx,
            out=Path("-"),
            quiet=True,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )

    assert exc.value.exit_code == 0
    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


def test_docs_stdout_yaml(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that YAML output is correctly echoed to stdout."""
    monkeypatch.setenv("BIJUXCLI_DOCS_OUT", "")
    monkeypatch.setattr(docs_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(
        docs_mod, "resolve_command_config", _fake_resolve_command_config
    )
    monkeypatch.setattr(docs_mod, "_build_spec_payload", lambda ir: {"hello": "world"})

    monkeypatch.setattr(
        docs_mod,
        "_resolve_docs_service",
        lambda: FakeDocsService(render_value="{hello: world}\n"),
    )

    ctx: Context = MagicMock()
    ctx.invoked_subcommand = None
    ctx.args = []
    with pytest.raises(Exit) as exc:
        docs(
            ctx,
            out=Path("-"),
            quiet=False,
            verbose=False,
            fmt="yaml",
            pretty=False,
            log_level="info",
        )

    assert exc.value.exit_code == 0
    out, err = capsys.readouterr()
    assert out.strip() == "{hello: world}"
    assert err == ""


@patch("bijux_cli.cli.commands.diagnostics.docs.emit_error_and_exit", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.resolve_command_config", autospec=True)
@patch("bijux_cli.cli.commands.diagnostics.docs.contains_non_ascii_env", autospec=True)
def test_docs_yaml_serialization_failure(
    mock_nonascii: MagicMock,
    mock_resolve: MagicMock,
    mock_emit: MagicMock,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a YAML serialization failure is handled gracefully."""
    mock_nonascii.return_value = False
    mock_resolve.side_effect = lambda **_kw: (
        ExecutionPolicy(
            output_format="yaml",
            color=ColorMode.AUTO,
            quiet=False,
            verbose=False,
            verbose_level=0,
            log_level="info",
            pretty=True,
            include_runtime=False,
            json=False,
        ),
        OutputFormat.YAML,
        "yaml",
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
        docs(
            ctx,
            out=None,
            quiet=False,
            verbose=False,
            fmt="yaml",
            pretty=True,
            log_level="info",
        )

    mock_emit.assert_called_once_with(
        "Serialization failed: yaml‐oops",
        code=1,
        failure="serialize",
        command="docs",
        fmt="yaml",
        quiet=False,
        include_runtime=False,
        debug=False,
    )
