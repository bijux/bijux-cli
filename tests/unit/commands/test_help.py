# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the help command."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

from collections.abc import Iterator
import importlib
import sys
import time
from typing import Any
from unittest.mock import MagicMock

import click
import pytest
import typer

import bijux_cli.commands.help as help_mod
from bijux_cli.commands.help import _HUMAN, _build_help_payload, _find_target_command
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import OutputFormat


class DummyCmd(click.Command):
    """A dummy click.Command with injectable help."""

    def __init__(
        self, name: str, help_text: str, context_settings: dict[str, Any] | None = None
    ) -> None:
        """Initialize the dummy command."""
        super().__init__(
            name=name, callback=lambda: None, context_settings=context_settings or {}
        )
        self._help_text = help_text

    def get_help(self, ctx: click.Context) -> str:
        """Return the injectable help text."""
        return self._help_text


@pytest.fixture(autouse=True)
def _cleanup_help_module_patches() -> Iterator[None]:  # pyright: ignore[reportUnusedFunction]
    """A fixture to automatically clean up the global patches made by help_mod."""
    yield

    click.echo = _real_click_echo
    click.secho = _real_click_secho
    typer.echo = _real_typer_echo
    typer.secho = _real_typer_secho
    sys.stderr.write = _real_stderr_write  # type: ignore[method-assign]


def test_find_target_command_no_parent() -> None:
    """Test that finding a target command fails with no parent context."""
    ctx0 = typer.Context(click.Command("dummy"))
    assert _find_target_command(ctx0, ["anything"]) is None


def test_find_target_command_empty_path() -> None:
    """Test that an empty path resolves to the root command."""
    root = click.Group(name="root")
    parent_ctx = typer.Context(root, info_name="root")
    ctx = typer.Context(root, parent=parent_ctx)
    found = _find_target_command(ctx, [])
    assert found is not None
    cmd, cmd_ctx = found
    assert cmd is root
    assert isinstance(cmd_ctx, click.Context)
    assert cmd_ctx.command is root


def test_find_target_command_not_found() -> None:
    """Test that a non-existent command path returns None."""
    root = click.Group(name="root")
    root.add_command(click.Command("foo"))
    parent_ctx = typer.Context(root, info_name="root")
    ctx = typer.Context(root, parent=parent_ctx)
    assert _find_target_command(ctx, ["bar"]) is None


def test_get_formatted_help_replaces_help_option() -> None:
    """Test that the short help option is replaced with the standard one."""
    dummy = DummyCmd(
        "x", "--help usage", context_settings={"help_option_names": ["-h"]}
    )
    ctx = typer.Context(dummy, info_name="x")
    out = help_mod._get_formatted_help(dummy, ctx)
    assert "-h, --help" in out


def test_get_formatted_help_no_change() -> None:
    """Test that help text is unchanged if standard help option is used."""
    dummy = DummyCmd(
        "x", "--help usage", context_settings={"help_option_names": ["--help"]}
    )
    ctx = typer.Context(dummy, info_name="x")
    out = help_mod._get_formatted_help(dummy, ctx)
    assert out == "--help usage"


def test_build_help_payload_without_runtime() -> None:
    """Test building a help payload without runtime info."""
    start = time.perf_counter()
    p = _build_help_payload("txt", False, start)
    assert p == {"help": "txt"}


def test_build_help_payload_with_runtime(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test building a help payload with runtime info."""
    monkeypatch.setattr(help_mod.time, "perf_counter", lambda: 1000.0)  # type: ignore[attr-defined]
    p = _build_help_payload("txt", True, started_at=999.0)
    assert p["help"] == "txt"
    assert "python" in p
    assert "platform" in p
    assert "runtime_ms" in p
    assert isinstance(p["runtime_ms"], int)


def make_ctx_for_callback(
    tmp_group: click.Group | None = None,
) -> typer.Context:
    """Build a Typer Context suitable for passing to help_callback."""
    group = tmp_group or click.Group(name="root")
    parent_ctx = typer.Context(group, info_name="root")
    ctx = typer.Context(group, parent=parent_ctx)
    return ctx


def test_help_flag_triggers_help_and_exit(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that the --help flag on the help command prints help and exits."""
    monkeypatch.setattr(sys, "argv", ["prog", "help", "--help", "foo"])
    group = click.Group(name="root")
    foo_cmd = DummyCmd("foo", "FOO HELP")
    group.add_command(foo_cmd)
    parent_ctx = typer.Context(group, info_name="bijux")
    ctx = typer.Context(foo_cmd, info_name="foo", parent=parent_ctx)
    with pytest.raises(typer.Exit) as ex:
        help_mod.help_callback(
            ctx, None, quiet=False, verbose=False, fmt=_HUMAN, pretty=True, debug=False
        )
    captured = capsys.readouterr()
    assert "FOO HELP" in captured.out
    assert ex.value.exit_code == 0


def test_quiet_invalid_format(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that quiet mode with an invalid format exits with code 2."""
    ctx = make_ctx_for_callback()
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx, [], quiet=True, verbose=False, fmt="badfmt", pretty=True, debug=False
        )
    assert ex.value.code == 2


def test_quiet_null_byte(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that quiet mode with a null byte in tokens exits with code 3."""
    ctx = make_ctx_for_callback()
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx,
            ["\x00foo"],
            quiet=True,
            verbose=False,
            fmt=_HUMAN,
            pretty=True,
            debug=False,
        )
    assert ex.value.code == 3


def test_quiet_non_ascii_token(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that quiet mode with a non-ASCII token exits with code 3."""
    ctx = make_ctx_for_callback()
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx,
            ["föo"],
            quiet=True,
            verbose=False,
            fmt=_HUMAN,
            pretty=True,
            debug=False,
        )
    assert ex.value.code == 3


def test_quiet_non_ascii_env(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that quiet mode with a non-ASCII env var exits with code 3."""
    ctx = make_ctx_for_callback()
    monkeypatch.setattr(help_mod, "contains_non_ascii_env", lambda: True)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx, [], quiet=True, verbose=False, fmt=_HUMAN, pretty=True, debug=False
        )
    assert ex.value.code == 3


def test_quiet_not_found(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that quiet mode with a non-existent target exits with code 2."""
    ctx = make_ctx_for_callback()
    monkeypatch.setattr(help_mod, "_find_target_command", lambda c, p: None)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx, [], quiet=True, verbose=False, fmt=_HUMAN, pretty=True, debug=False
        )
    assert ex.value.code == 2


def test_nonquiet_invalid_format_calls_emit_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that non-quiet mode with invalid format emits a structured error."""
    ctx = make_ctx_for_callback()
    monkeypatch.setattr(help_mod, "validate_common_flags", lambda *a, **k: None)
    called: dict[str, Any] = {}

    def fake_error(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
    ) -> None:
        called.update(locals())
        raise SystemExit(code)

    monkeypatch.setattr(help_mod, "emit_error_and_exit", fake_error)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx, [], quiet=False, verbose=False, fmt="BAD", pretty=True, debug=False
        )
    assert ex.value.code == 2
    assert "Unsupported format" in called["msg"]
    assert called["fmt"] == "json"


def test_nonquiet_null_byte_emits_null_byte_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that non-quiet mode with a null byte emits a structured error."""
    ctx = make_ctx_for_callback()
    monkeypatch.setattr(help_mod, "validate_common_flags", lambda *a, **k: None)
    called: dict[str, Any] = {}

    def fake_error(msg: str, code: int, failure: str, **kwargs: Any) -> None:
        called.update(locals())
        raise SystemExit(code)

    monkeypatch.setattr(help_mod, "emit_error_and_exit", fake_error)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx,
            ["\x00"],
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
        )
    assert ex.value.code == 3
    assert called["failure"] == "null_byte"


def test_nonquiet_nonascii_token_emits_ascii_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that non-quiet mode with a non-ASCII token emits a structured error."""
    ctx = make_ctx_for_callback()
    monkeypatch.setattr(help_mod, "validate_common_flags", lambda *a, **k: None)
    called: dict[str, Any] = {}

    def fake_error(msg: str, code: int, failure: str, **kwargs: Any) -> None:
        called.update(locals())
        raise SystemExit(code)

    monkeypatch.setattr(help_mod, "emit_error_and_exit", fake_error)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx,
            ["föo"],
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
        )
    assert ex.value.code == 3
    assert called["failure"] == "ascii"


def test_nonquiet_ascii_env_emits_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that non-quiet mode with a non-ASCII env emits a structured error."""
    ctx = make_ctx_for_callback()
    monkeypatch.setattr(help_mod, "validate_common_flags", lambda *a, **k: None)
    monkeypatch.setattr(help_mod, "contains_non_ascii_env", lambda: True)
    called: dict[str, Any] = {}

    def fake_error(msg: str, code: int, failure: str, **kwargs: Any) -> None:
        called.update(locals())
        raise SystemExit(code)

    monkeypatch.setattr(help_mod, "emit_error_and_exit", fake_error)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx, [], quiet=False, verbose=False, fmt="json", pretty=True, debug=False
        )
    assert ex.value.code == 3
    assert called["failure"] == "ascii"


def test_nonquiet_not_found_emits_not_found(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that non-quiet mode with a non-existent target emits a not_found error."""
    ctx = make_ctx_for_callback()
    monkeypatch.setattr(help_mod, "validate_common_flags", lambda *a, **k: None)
    monkeypatch.setattr(help_mod, "contains_non_ascii_env", lambda: False)
    monkeypatch.setattr(help_mod, "_find_target_command", lambda c, p: None)
    called: dict[str, Any] = {}

    def fake_error(msg: str, code: int, failure: str, **kwargs: Any) -> None:
        called.update(locals())
        raise SystemExit(code)

    monkeypatch.setattr(help_mod, "emit_error_and_exit", fake_error)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx,
            ["no", "such"],
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
        )
    assert ex.value.code == 2
    assert called["failure"] == "not_found"


def test_nonquiet_human_format_prints_and_exits(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that human format prints help text to stdout and exits."""
    group = click.Group(name="root")
    foo = DummyCmd("foo", "X HELP")
    group.add_command(foo)
    parent_ctx = typer.Context(group, info_name="bijux")
    ctx = typer.Context(group, parent=parent_ctx)
    monkeypatch.setattr(
        help_mod, "_find_target_command", lambda c, p: (foo, typer.Context(foo))
    )

    class DummyContainer:
        def resolve(self, proto: type) -> None:
            return None

    monkeypatch.setattr(
        DIContainer, "current", classmethod(lambda cls: DummyContainer())
    )

    with pytest.raises(typer.Exit) as ex:
        help_mod.help_callback(
            ctx,
            ["foo"],
            quiet=False,
            verbose=False,
            fmt="human",
            pretty=False,
            debug=False,
        )

    out = capsys.readouterr().out
    assert "X HELP" in out
    assert ex.value.exit_code == 0


def test_nonquiet_json_format_emits_payload(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that JSON format emits a structured payload."""
    group = click.Group(name="root")
    foo = DummyCmd("foo", "HELPTXT")
    group.add_command(foo)
    parent = typer.Context(group, info_name="bijux")
    ctx = typer.Context(group, parent=parent)
    monkeypatch.setattr(
        help_mod, "_find_target_command", lambda c, p: (foo, typer.Context(foo))
    )
    monkeypatch.setattr(
        DIContainer,
        "current",
        classmethod(lambda cls: MagicMock(resolve=lambda p: None)),
    )
    called: dict[str, Any] = {}

    def fake_emit(
        payload: dict[str, Any],
        fmt: OutputFormat,
        effective_pretty: bool,
        verbose: bool,
        debug: bool,
        quiet: bool,
        command: str,
        exit_code: int,
    ) -> None:
        called.update(locals())
        raise SystemExit(exit_code)

    monkeypatch.setattr(help_mod, "emit_and_exit", fake_emit)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx,
            ["foo"],
            quiet=False,
            verbose=True,
            fmt="json",
            pretty=False,
            debug=False,
        )
    assert ex.value.code == 0
    assert called["payload"]["help"] == "HELPTXT"
    assert "python" in called["payload"]
    assert "platform" in called["payload"]
    assert "runtime_ms" in called["payload"]
    assert called["fmt"] is OutputFormat.JSON


def test_nonquiet_yaml_format_emits_payload(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that YAML format emits a structured payload."""
    group = click.Group(name="root")
    foo = DummyCmd("foo", "YAMLHELP")
    group.add_command(foo)
    parent = typer.Context(group, info_name="bijux")
    ctx = typer.Context(group, parent=parent)
    monkeypatch.setattr(
        help_mod, "_find_target_command", lambda c, p: (foo, typer.Context(foo))
    )
    monkeypatch.setattr(
        DIContainer,
        "current",
        classmethod(lambda cls: MagicMock(resolve=lambda p: None)),
    )
    called: dict[str, Any] = {}

    def fake_emit(payload: dict[str, Any], fmt: OutputFormat, **kwargs: Any) -> None:
        called.update(locals())
        raise SystemExit(99)

    monkeypatch.setattr(help_mod, "emit_and_exit", fake_emit)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx,
            ["foo"],
            quiet=False,
            verbose=False,
            fmt="yaml",
            pretty=True,
            debug=False,
        )
    assert ex.value.code == 99
    assert called["payload"]["help"] == "YAMLHELP"
    assert called["fmt"] is OutputFormat.YAML


def test_find_target_command_path_too_long() -> None:
    """Test that a path deeper than the command structure returns None."""
    root = click.Group(name="root")
    root.add_command(click.Command("foo"))
    parent_ctx = typer.Context(root, info_name="root")
    ctx = typer.Context(root, parent=parent_ctx)
    assert _find_target_command(ctx, ["foo", "bar"]) is None


def test_quiet_success(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that quiet mode with a valid target exits with code 0."""
    ctx = make_ctx_for_callback()
    monkeypatch.setattr(
        help_mod, "_find_target_command", lambda c, p: (object(), object())
    )
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx, [], quiet=True, verbose=False, fmt=_HUMAN, pretty=True, debug=False
        )
    assert ex.value.code == 0


def test_payload_value_error_emits_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an error during payload building emits a structured error."""
    group = click.Group(name="root")
    foo = DummyCmd("foo", "whatever")
    group.add_command(foo)
    parent = typer.Context(group, info_name="bijux")
    ctx = typer.Context(group, parent=parent)
    monkeypatch.setattr(
        help_mod, "_find_target_command", lambda c, p: (foo, typer.Context(foo))
    )
    monkeypatch.setattr(
        DIContainer,
        "current",
        classmethod(lambda cls: MagicMock(resolve=lambda p: None)),
    )
    monkeypatch.setattr(
        help_mod,
        "_build_help_payload",
        lambda *args, **kwargs: (_ for _ in ()).throw(ValueError("broken")),
    )
    called: dict[str, Any] = {}

    def fake_emit(msg: str, code: int, failure: str, **kwargs: Any) -> None:
        called.update(locals())
        raise SystemExit(code)

    monkeypatch.setattr(help_mod, "emit_error_and_exit", fake_emit)
    with pytest.raises(SystemExit) as ex:
        help_mod.help_callback(
            ctx,
            ["foo"],
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            debug=False,
        )
    assert ex.value.code == 3
    assert "broken" in called["msg"]
    assert called["failure"] == "ascii"


def test_import_level_overrides(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test the module-level filtering of stderr and echo functions."""
    import sys as real_sys

    import click as real_click
    import typer as real_typer

    calls: list[str] = []

    def dummy_write(data: str) -> int:
        calls.append(data)
        return len(data)

    monkeypatch.setattr(real_sys.stderr, "write", dummy_write)

    orig_click_echo = real_click.echo
    orig_click_secho = real_click.secho
    orig_typer_echo = real_typer.echo
    orig_typer_secho = real_typer.secho

    monkeypatch.setattr(real_sys, "argv", ["prog", "help", "--quiet"])
    importlib.reload(help_mod)

    assert real_sys.stderr.write("") == 0
    assert real_sys.stderr.write("   ") == 0

    plugin_msg = "Plugin 'test-src' does not expose a Typer app via 'cli()' or 'app'"
    assert real_sys.stderr.write(plugin_msg) == 0

    assert real_sys.stderr.write("hello") == dummy_write("hello")

    real_click.echo("")
    real_click.echo("   ")
    real_click.echo("[WARN] Plugin 'test-src' does not expose a Typer app")
    real_click.echo("hi")
    out = capsys.readouterr().out
    assert "hi" in out

    real_click.echo = orig_click_echo
    real_click.secho = orig_click_secho
    real_typer.echo = orig_typer_echo
    real_typer.secho = orig_typer_secho


def test_help_flag_no_target(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that --help for a non-existent target exits cleanly with no output."""
    monkeypatch.setattr(sys, "argv", ["prog", "help", "--help", "nope"])
    group = click.Group(name="root")
    parent_ctx = typer.Context(group, info_name="bijux")
    ctx = typer.Context(group, parent=parent_ctx)
    monkeypatch.setattr(help_mod, "_find_target_command", lambda c, p: None)
    with pytest.raises(typer.Exit) as ex:
        help_mod.help_callback(
            ctx, None, quiet=False, verbose=False, fmt=_HUMAN, pretty=True, debug=False
        )
    assert ex.value.exit_code == 0
    assert capsys.readouterr().out == ""


def test_help_flag_fallback_to_root(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that --help with no target prints help for the root command."""
    monkeypatch.setattr(sys, "argv", ["prog", "help", "--help"])

    class FakeGroup(click.Group):
        def get_help(self, ctx: click.Context) -> str:
            return "ROOT HELP"

    group = FakeGroup(name="root")
    parent_ctx = typer.Context(group, info_name="bijux")
    ctx = typer.Context(group, parent=parent_ctx)
    monkeypatch.setattr(
        help_mod,
        "_find_target_command",
        lambda c, p: None if p else (group, parent_ctx),
    )
    with pytest.raises(typer.Exit) as exc:
        help_mod.help_callback(
            ctx, None, quiet=False, verbose=False, fmt=_HUMAN, pretty=True, debug=False
        )
    out = capsys.readouterr().out
    assert "ROOT HELP" in out
    assert exc.value.exit_code == 0


def test_help_flag_with_format_flag(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test that --help correctly parses command path when other flags are present."""
    monkeypatch.setattr(sys, "argv", ["prog", "help", "--help", "-f", "json", "foo"])
    group = click.Group(name="root")
    foo = DummyCmd("foo", "FOO HELP")
    group.add_command(foo)
    parent_ctx = typer.Context(group, info_name="bijux")
    ctx = typer.Context(group, parent=parent_ctx)
    monkeypatch.setattr(
        help_mod, "_find_target_command", lambda c, p: (foo, typer.Context(foo))
    )
    with pytest.raises(typer.Exit) as exc:
        help_mod.help_callback(
            ctx, None, quiet=False, verbose=False, fmt=_HUMAN, pretty=True, debug=False
        )
    out = capsys.readouterr().out
    assert "FOO HELP" in out
    assert exc.value.exit_code == 0


_real_click_echo = click.echo
_real_click_secho = click.secho
_real_typer_echo = typer.echo
_real_typer_secho = typer.secho
_real_stderr_write = sys.stderr.write


def test_help_module_level_filter_and_restore(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the import-time patching and subsequent restoration of echo/write."""
    monkeypatch.setattr(sys, "argv", ["prog", "help", "--quiet"])
    filtered = importlib.reload(help_mod)

    assert click.echo is filtered._filtered_echo
    assert click.secho is filtered._filtered_echo
    assert typer.echo is filtered._filtered_echo
    assert typer.secho is filtered._filtered_echo

    monkeypatch.setattr(click, "echo", _real_click_echo)
    monkeypatch.setattr(click, "secho", _real_click_secho)
    monkeypatch.setattr(typer, "echo", _real_typer_echo)
    monkeypatch.setattr(typer, "secho", _real_typer_secho)
    monkeypatch.setattr(sys.stderr, "write", _real_stderr_write)

    assert click.echo is _real_click_echo
    assert click.secho is _real_click_secho
    assert typer.echo is _real_typer_echo
    assert typer.secho is _real_typer_secho
    assert sys.stderr.write is _real_stderr_write


def test_filtered_echo_non_str_message(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the filtered echo function correctly handles non-string messages."""
    monkeypatch.setattr(sys, "argv", ["prog", "help", "--quiet"])
    filtered = importlib.reload(help_mod)
    orig_echo = filtered._orig_click_echo
    called = False

    def fake_orig(message: Any, **kwargs: Any) -> None:
        nonlocal called
        called = True
        assert message == b"bytes"

    monkeypatch.setattr(filtered, "_orig_click_echo", fake_orig)
    filtered._filtered_echo(b"bytes")
    assert called

    click.echo = orig_echo
    click.secho = orig_echo
    typer.echo = orig_echo
    typer.secho = orig_echo
    if hasattr(filtered, "_orig_stderr_write"):
        monkeypatch.setattr(sys.stderr, "write", filtered._orig_stderr_write)
