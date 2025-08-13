# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the repl command."""

from __future__ import annotations

import asyncio
from collections.abc import Callable
import io
import json
from pathlib import Path
import signal
import sys
import types
from typing import Any, cast
from unittest.mock import patch

from prompt_toolkit.document import Document
import pytest
import typer

import bijux_cli.commands.repl as mod

# pytype: disable=invalid-annotation
# pytype: disable=name-error


class _FakeResult:
    """Fake result object resembling Typer's runner result."""

    def __init__(self, code: int = 0, stdout: str = "", stderr: str = "") -> None:
        """Init."""
        self.exit_code: int = code
        self.stdout: str = stdout
        self.stderr: str = stderr


class _RecordingFakeRunner:
    """CliRunner replacement that records invocations."""

    def __init__(self, recorder: list[list[str]] | None = None) -> None:
        """Init."""
        self.recorder: list[list[str]] | None = recorder

    def invoke(
        self, _app: Any, tokens: list[str], env: dict[str, str] | None = None
    ) -> _FakeResult:  # noqa: ARG002
        """Invoke a fake CLI command."""
        if self.recorder is not None:
            self.recorder.append(list(tokens))
        if tokens and tokens[0] == "version":
            return _FakeResult(0, '{"version":"0.0.0"}\n', "")
        if tokens[:2] == ["config", "list"]:
            return _FakeResult(0, "CONFIG-LIST-COMPACT\n", "")
        if tokens and tokens[0] == "history":
            return _FakeResult(0, '{"entries": []}\n', "")
        if tokens[:2] == ["memory", "list"]:
            return _FakeResult(0, '{"entries":["x"]}\n', "")
        if tokens and tokens[0] == "status":
            return _FakeResult(0, json.dumps({"status": "ok"}) + "\n", "")
        return _FakeResult(0, "", "")


def _capture_io() -> tuple[io.StringIO, io.StringIO, pytest.MonkeyPatch]:
    """Capture stdout and stderr and return patch handle."""
    out, err = io.StringIO(), io.StringIO()
    mp = pytest.MonkeyPatch()
    mp.setattr(sys, "stdout", out)
    mp.setattr(sys, "stderr", err)
    return out, err, mp


def test_filter_control_removes_ansi() -> None:
    """Remove ANSI escapes from text."""
    s = "\x1b[31mred\x1b[0m plain"
    assert mod._filter_control(s) == "red plain"  # pyright: ignore[reportPrivateUsage]


def test_split_segments_handles_quotes_and_semicolons() -> None:
    """Split segments respecting quotes and semicolons."""
    text = "a; b\n'c; d'; \"e;f\" ;  ; g"
    assert list(mod._split_segments(text)) == [  # pyright: ignore[reportPrivateUsage]
        "a",
        "b",
        "'c; d'",
        '"e;f"',
        "g",
    ]


def test_suggest() -> None:
    """Suggest nearest known command."""
    orig = mod._known_commands  # pyright: ignore[reportPrivateUsage]
    mod._known_commands = lambda: [  # pyright: ignore[reportPrivateUsage]
        "status",
        "version",
    ]
    try:
        hint = mod._suggest("verzion")  # pyright: ignore[reportPrivateUsage]
        assert hint
        assert "version" in hint
        assert mod._suggest("version") is None  # pyright: ignore[reportPrivateUsage]
        assert mod._suggest("zzzz") is None  # pyright: ignore[reportPrivateUsage]
    finally:
        mod._known_commands = orig  # pyright: ignore[reportPrivateUsage]


def build_fake_root_app() -> Any:
    """Build a small Typer app used by tests."""
    import typer

    app = typer.Typer()

    @app.command()
    def status(  # pyright: ignore[reportUnusedFunction]
        quiet: bool = typer.Option(False, "-q", "--quiet"),
        pretty: bool = typer.Option(True, "--pretty/--no-pretty"),
        fmt: str = typer.Option("json", "-f", "--format"),
    ) -> None:  # noqa: ARG001
        payload = {"status": "ok"}
        if not quiet:
            txt = json.dumps(payload, indent=2 if pretty else None)
            typer.echo(txt)

    @app.command()
    def history(  # noqa: ARG001 # pyright: ignore[reportUnusedFunction]
        pretty: bool = typer.Option(True, "--pretty/--no-pretty"),
        fmt: str = typer.Option("json", "-f", "--format"),
    ) -> None:
        typer.echo('{"entries": []}')

    return app


def _install_fake_cli_module(app: Any) -> None:
    """Install a module named bijux_cli.cli exposing `app`."""
    fake = types.ModuleType("bijux_cli.cli")
    fake.app = app  # type: ignore[attr-defined]
    sys.modules["bijux_cli.cli"] = fake


def _run_piped_lines(lines: list[str], *, quiet: bool = False) -> None:
    """Feed lines to stdin and run _run_piped()."""
    buf = io.StringIO("\n".join(lines))
    with patch.object(sys, "stdin", buf):
        with pytest.raises(SystemExit) as ex:
            mod._run_piped(quiet)  # pyright: ignore[reportPrivateUsage]
        assert ex.value.code == 0


def test_run_piped_flow_and_messages(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Exercise piped flow and messages."""
    monkeypatch.setattr(
        mod, "_known_commands", lambda: ["status", "memory", "config", "docs"]
    )
    calls: list[list[str]] = []

    def fake_invoke(tokens: list[str], repl_quiet: bool) -> int:
        calls.append(tokens)
        return 0

    monkeypatch.setattr(mod, "_invoke", fake_invoke)
    monkeypatch.setattr(
        mod,
        "_suggest",
        lambda s: (  # pyright: ignore[reportUnknownLambdaType]
            " Did you mean 'status'?" if s != "status" else None
        ),
    )

    _run_piped_lines(
        [
            "",
            "# comment",
            ";bad",
            "docs",
            "docs topicX",
            "memory list",
            "-unknown",
            "config set",
            "config get",
            "unknown",
            "status -q",
            "  quit  ",
        ],
        quiet=False,
    )

    out = capsys.readouterr()
    assert "Available topics: …" in out.out
    assert "topicX" in out.out
    assert '"failure": "missing_argument"' in out.out
    assert ["memory", "list"] in calls
    assert ["status", "-q"] in calls
    assert "No such command 'bad'. Did you mean 'status'?" in (out.err or out.out)
    assert "No such command 'unknown'. Did you mean 'status'?" in (out.err or out.out)


def test_run_piped_quiet_suppresses_everything(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Suppress all output in quiet mode."""
    monkeypatch.setattr(mod, "_known_commands", lambda: ["status"])
    monkeypatch.setattr(
        mod,
        "_suggest",
        lambda s: None,  # pyright: ignore[reportUnknownLambdaType]
    )
    monkeypatch.setattr(
        mod,
        "_invoke",
        lambda *a, **k: 0,  # pyright: ignore[reportUnknownLambdaType]
    )
    _run_piped_lines(["", "# x", ";bad", "-flag", "config set"], quiet=True)
    out = capsys.readouterr()
    assert out.out == ""
    assert out.err == ""


def test_get_prompt_plain_and_ansi(monkeypatch: pytest.MonkeyPatch) -> None:
    """Return plain prompt under test/NO_COLOR, otherwise ANSI object."""
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "1")
    assert mod.get_prompt() == "bijux> "
    monkeypatch.delenv("BIJUXCLI_TEST_MODE", raising=False)
    monkeypatch.setenv("NO_COLOR", "1")
    assert mod.get_prompt() == "bijux> "
    monkeypatch.delenv("NO_COLOR", raising=False)
    p = mod.get_prompt()
    assert "bijux" in str(p)


def test_exit_on_signal() -> None:
    """Exit cleanly on signal."""
    with pytest.raises(SystemExit) as ex:
        mod._exit_on_signal(2, None)  # pyright: ignore[reportPrivateUsage]
    assert ex.value.code == 0


class FakeParam:
    """Fake Typer parameter descriptor."""

    def __init__(
        self,
        opts: list[str] | tuple[str, ...],
        secondary_opts: list[str] | tuple[str, ...] = (),
    ) -> None:
        """Init."""
        self.opts: tuple[str, ...] = tuple(opts)
        self.secondary_opts: tuple[str, ...] = tuple(secondary_opts)


class FakeCmd:
    """Fake Typer command descriptor."""

    def __init__(self, name: str, params: list[FakeParam] | None = None) -> None:
        """Init."""
        self.name: str = name
        self.params: list[FakeParam] = params or []


class FakeGroup:
    """Fake Typer group descriptor."""

    def __init__(self, name: str, app: FakeApp) -> None:
        """Init."""
        self.name: str = name
        self.typer_instance: FakeApp = app


class FakeApp:
    """Fake Typer app descriptor."""

    def __init__(self, commands: list[FakeCmd], groups: list[FakeGroup]) -> None:
        """Init."""
        self.registered_commands: list[FakeCmd] = commands
        self.registered_groups: list[FakeGroup] = groups


def make_fake_completer() -> mod.CommandCompleter:
    """Construct a CommandCompleter using fake app graph."""
    status = FakeCmd("status", [FakeParam(["--no-pretty", "--pretty"])])
    version = FakeCmd("version")
    config_cmds = [FakeCmd("set", [])]
    config_app = FakeApp(config_cmds, [])
    config_group = FakeGroup("config", config_app)
    app = FakeApp([status, version], [config_group])
    return mod.CommandCompleter(app)  # type: ignore[arg-type]


def complete_text(comp: mod.CommandCompleter, text: str) -> list[str]:
    """Return completion texts for input."""
    ev = object()
    return [c.text for c in comp.get_completions(Document(text), ev)]  # type: ignore[arg-type]


def test_completer_global_opts_builtins_and_top() -> None:
    """Complete global opts, built-ins, and top-level commands."""
    comp = make_fake_completer()
    assert "--no-pretty" in complete_text(comp, "--no")
    assert "exit" in complete_text(comp, "ex")
    assert "status" in complete_text(comp, "s")


class FakePromptSession:
    """Fake PromptSession that returns pre-defined lines."""

    def __init__(self, *a: Any, **k: Any) -> None:
        """Init."""
        self._lines: list[str] = [
            "",
            "# comment",
            "docs",
            "docs topicA",
            "memory list",
            "unknown",
            "status",
            "quit",
        ]
        self._i: int = 0

    async def prompt_async(self) -> str:
        """Return next line or raise EOFError."""
        if self._i >= len(self._lines):
            raise EOFError
        v = self._lines[self._i]
        self._i += 1
        return v


def test_main_human_quiet_routes_to_piped(monkeypatch: pytest.MonkeyPatch) -> None:
    """Route human quiet mode to piped path."""
    monkeypatch.setattr(
        signal,
        "signal",
        lambda *a, **k: None,  # pyright: ignore[reportUnknownLambdaType]
    )
    monkeypatch.setattr(sys.stdin, "isatty", lambda: False)
    flag: dict[str, bool] = {}

    def fake_run_piped(q: bool) -> None:
        flag["q"] = q
        raise SystemExit(0)

    monkeypatch.setattr(mod, "_run_piped", fake_run_piped)
    from typer.testing import CliRunner

    res = CliRunner().invoke(mod.repl_app, ["--quiet"])
    assert res.exit_code == 0
    assert flag["q"] is True


def test_main_invalid_format_emits_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Emit error when invalid format is provided."""
    seen: dict[str, Any] = {"validate": None, "emit": None}

    def vcf(fmt: str, command: str, quiet: bool, include_runtime: bool) -> str:
        seen["validate"] = (fmt, command, quiet, include_runtime)
        return fmt

    def emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
    ) -> None:  # noqa: ARG001
        seen["emit"] = (msg, code, failure, command, fmt, quiet, include_runtime, debug)
        raise SystemExit(2)

    monkeypatch.setattr(mod, "validate_common_flags", vcf)
    monkeypatch.setattr(mod, "emit_error_and_exit", emit)
    monkeypatch.setattr(
        sys.stdin,
        "isatty",
        lambda: False,  # pyright: ignore[reportUnknownLambdaType]
    )
    monkeypatch.setattr(
        signal,
        "signal",
        lambda *a, **k: None,  # pyright: ignore[reportUnknownLambdaType]
    )
    from typer.testing import CliRunner

    res = CliRunner().invoke(mod.repl_app, ["--format", "yaml"])
    assert res.exit_code == 2
    assert cast(tuple[str, str, bool, bool], seen["validate"])[0] == "yaml"
    assert cast(tuple[str, int, str, str, str, bool, bool, bool], seen["emit"])[1] == 2


def test_known_commands_spec_invalid_json_falls_back(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fallback when spec contains invalid JSON."""

    class FakePath:
        def __init__(self, path: str = "") -> None:
            self._path = path

        def resolve(self) -> FakePath:
            return self

        @property
        def parent(self) -> FakePath:
            return self

        @property
        def parents(self) -> list[FakePath]:
            return [self]

        def is_file(self) -> bool:
            return self._path.endswith("spec.json")

        def read_text(self, *a: object, **k: object) -> str:
            return "{"

        def __truediv__(self, other: str) -> FakePath:
            return FakePath(f"{self._path}/{other}")

        def __str__(self) -> str:
            return self._path

    monkeypatch.setattr(
        mod,
        "Path",
        lambda *_: FakePath("dummy"),  # pyright: ignore[reportUnknownLambdaType]
    )
    cmds = mod._known_commands()  # pyright: ignore[reportPrivateUsage]
    assert {"status", "version", "repl"} <= set(cmds)


def test_invoke_history_prints_empty_entries_pretty(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Pretty-print history with empty entries."""
    _install_fake_cli_module(build_fake_root_app())
    out, err, m = _capture_io()
    try:
        code = mod._invoke(  # pyright: ignore[reportPrivateUsage]
            ["history"], repl_quiet=False
        )
        assert code == 0
        s = out.getvalue().strip()
        assert s.startswith("{")
        assert '"entries": []' in s
        assert err.getvalue() == ""
    finally:
        m.undo()


def make_real_completer() -> mod.CommandCompleter:
    """Build a real Typer app and wrap in completer."""
    import typer

    app = typer.Typer()

    @app.command()
    def status(  # noqa: D401 # pyright: ignore[reportUnusedFunction]
        no_pretty: bool = typer.Option(False, "--no-pretty"),
        pretty: bool = typer.Option(False, "--pretty"),
    ) -> None:
        """Status command."""
        return None

    config = typer.Typer()

    @config.command("set")
    def config_set(  # pyright: ignore[reportUnusedFunction]
        arg: str = typer.Argument(""),
    ) -> None:  # noqa: D401, ARG001
        """Config set command."""
        return None

    app.add_typer(config, name="config")
    return mod.CommandCompleter(app)


@pytest.mark.asyncio
async def test_run_interactive_end_to_end(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Exercise interactive REPL end-to-end."""
    status = FakeCmd("status", [FakeParam(["--no-pretty", "--pretty"])])
    config_app = FakeApp([FakeCmd("set", [])], [])
    config_group = FakeGroup("config", config_app)
    app = FakeApp([status], [config_group])

    import importlib

    fake_cli_mod = types.SimpleNamespace(build_app=lambda: app)
    monkeypatch.setattr(
        importlib,
        "import_module",
        lambda name: fake_cli_mod,  # pyright: ignore[reportUnknownLambdaType]
    )

    import prompt_toolkit

    monkeypatch.setattr(prompt_toolkit, "PromptSession", FakePromptSession)

    import subprocess as _subp

    ran: dict[str, Any] = {}

    def fake_run(argv: list[str], env: dict[str, str] | None = None) -> Any:  # noqa: ARG001
        ran["args"] = argv
        return types.SimpleNamespace(returncode=0)

    monkeypatch.setattr(_subp, "run", fake_run)

    calls: list[list[str]] = []

    def _record_invoke(tokens: list[str], repl_quiet: bool) -> int:
        """Record invoked tokens and return success code."""
        calls.append(tokens)
        return 0

    monkeypatch.setattr(mod, "_invoke", _record_invoke)

    monkeypatch.setattr(
        mod, "_known_commands", lambda: ["status", "memory", "config", "docs"]
    )
    monkeypatch.setattr(
        mod,
        "_suggest",
        lambda s: (  # pyright: ignore[reportUnknownLambdaType]
            " Did you mean 'status'?" if s != "status" else None
        ),
    )

    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(Path.cwd() / ".tmp_history"))
    monkeypatch.setattr(sys, "argv", ["bijux"])

    await mod._run_interactive()  # pyright: ignore[reportPrivateUsage]

    out = capsys.readouterr()
    assert "Available topics" in out.out
    assert "topicA" in out.out
    assert "Exiting REPL." in out.out
    assert "No such command 'unknown'. Did you mean 'status'?" in (out.err or out.out)
    assert cast(list[str], ran["args"])[:3] == ["bijux", "memory", "list"]
    assert ["status"] in calls


def test_invoke_json_commands_and_quiet(monkeypatch: pytest.MonkeyPatch) -> None:
    """Invoke JSON commands and respect quiet."""
    import typer.testing as typer_testing

    monkeypatch.setattr(typer_testing, "CliRunner", _RecordingFakeRunner)

    out, err, m = _capture_io()  # pyright: ignore[reportPrivateUsage] # pyright: ignore[reportUnusedVariable]
    try:
        code = mod._invoke(  # pyright: ignore[reportPrivateUsage]
            ["version"], repl_quiet=False
        )
        assert code == 0
        assert out.getvalue().strip().startswith("{")

        out.truncate(0)
        out.seek(0)
        code = mod._invoke(  # pyright: ignore[reportPrivateUsage]
            ["status", "--quiet"], repl_quiet=False
        )
        assert code == 0
        assert out.getvalue() == ""

        out.truncate(0)
        out.seek(0)
        code = mod._invoke(  # pyright: ignore[reportPrivateUsage]
            ["status"], repl_quiet=True
        )
        assert code == 0
        assert out.getvalue() == ""
    finally:
        m.undo()


def test_invoke_config_list_forces_no_pretty(monkeypatch: pytest.MonkeyPatch) -> None:
    """Force --no-pretty for config list."""
    import typer.testing as typer_testing

    calls: list[list[str]] = []
    monkeypatch.setattr(typer_testing, "CliRunner", lambda: _RecordingFakeRunner(calls))
    out, _, m = _capture_io()
    try:
        assert (
            mod._invoke(  # pyright: ignore[reportPrivateUsage]
                ["config", "list"], repl_quiet=False
            )
            == 0
        )
        assert "CONFIG-LIST-COMPACT" in out.getvalue()
        assert any(c[:2] == ["config", "list"] and "--no-pretty" in c for c in calls)
    finally:
        m.undo()


def test_completer_subcommands_params_help_placeholder_and_dummy(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Complete group subcommands, params, help, and placeholders."""
    comp = make_fake_completer()
    monkeypatch.setattr(sys.modules["bijux_cli.commands.repl"].typer, "Typer", FakeApp)
    assert "set" in complete_text(comp, "config ")
    opts = complete_text(comp, "status --")
    assert "--no-pretty" in opts
    assert "--pretty" in opts
    assert "--help" in complete_text(comp, "status --he")
    res = complete_text(comp, "config set ")
    assert ("KEY=VALUE" in res) or ("--help" in res)
    items = complete_text(comp, "")
    assert "exit" in items
    assert "config" in items
    assert "status" in items
    assert "DUMMY" not in items


def test_known_commands_spec_wrong_type(monkeypatch: pytest.MonkeyPatch) -> None:
    """Fallback when spec commands is the wrong type."""

    class FakePath:
        def __init__(self, path: str = "") -> None:
            self._path = path

        def resolve(self) -> FakePath:
            return self

        @property
        def parent(self) -> FakePath:
            return self

        @property
        def parents(self) -> list[FakePath]:
            return [self]

        def is_file(self) -> bool:
            return self._path.endswith("spec.json")

        def read_text(self, *a: object, **k: object) -> str:
            return json.dumps({"commands": {"bad": "type"}})

        def __truediv__(self, other: str) -> FakePath:
            # Support pathlib-like division
            return FakePath(f"{self._path}/{other}")

        def __str__(self) -> str:
            return self._path

    monkeypatch.setattr(mod, "Path", lambda *_: FakePath("dummy"))  # pyright: ignore[reportUnknownLambdaType]
    cmds = mod._known_commands()  # pyright: ignore[reportPrivateUsage]
    assert "status" in cmds
    assert "version" in cmds


def test_invoke_history_non_empty_prints_raw(monkeypatch: pytest.MonkeyPatch) -> None:
    """Print history raw when entries exist."""
    import typer.testing as typer_testing

    class _Runner:
        def invoke(
            self, _app: Any, tokens: list[str], env: dict[str, str] | None = None
        ) -> _FakeResult:  # noqa: ARG002
            return _FakeResult(0, '{"entries":["y"]}\n', "")

    monkeypatch.setattr(typer_testing, "CliRunner", lambda: _Runner())
    out, err, mp = _capture_io()
    try:
        code = mod._invoke(  # pyright: ignore[reportPrivateUsage]
            ["history"], repl_quiet=False
        )
        assert code == 0
        assert '"y"' in out.getvalue()
        assert err.getvalue() == ""
    finally:
        mp.undo()


def test_run_piped_ignores_unclosed_quote(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Ignore unclosed quote ValueError."""
    monkeypatch.setattr(mod, "_known_commands", lambda: ["status"])
    monkeypatch.setattr(mod, "_suggest", lambda s: None)
    monkeypatch.setattr(mod, "_invoke", lambda *a, **k: 0)
    _run_piped_lines(['"unterminated'], quiet=False)
    out = capsys.readouterr()
    assert out.out == ""


def test_completer_handles_shlex_valueerror() -> None:
    """Handle ValueError during completion parsing."""
    comp = make_fake_completer()
    list(comp.get_completions(Document('unclosed "'), object()))  # type: ignore[arg-type]


def test_completer_find_longest_prefix() -> None:
    """Prefer the longest matching prefix in finder."""
    comp = make_fake_completer()
    obj, rem = comp._find(  # pyright: ignore[reportPrivateUsage]
        ["config", "set", "foo"]
    )
    assert getattr(obj, "name", None) == "set"
    assert rem == ["foo"]


def test_main_returns_early_when_subcommand(monkeypatch: pytest.MonkeyPatch) -> None:
    """Return early when context already has a subcommand."""
    from click import Command
    from typer import Context

    flags: dict[str, bool] = {}

    monkeypatch.setattr(mod, "_run_piped", lambda q: flags.setdefault("piped", True))

    monkeypatch.setattr(asyncio, "run", lambda coro: flags.setdefault("ran", True))

    dummy_command = Command(name="dummy")
    ctx = Context(command=dummy_command)
    ctx.invoked_subcommand = "anything"

    mod.main(ctx, quiet=False, verbose=False, fmt="human", pretty=True, debug=False)

    assert "piped" not in flags
    assert "ran" not in flags


def test_main_interactive_path_calls_async_loop(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Ensure interactive branch awaits coroutine."""
    import signal
    import sys

    from click import Command
    from typer import Context

    monkeypatch.setattr(sys.stdin, "isatty", lambda: True)
    monkeypatch.setattr(signal, "signal", lambda *a, **k: None)

    async def _stub() -> None:
        return None

    monkeypatch.setattr(mod, "_run_interactive", _stub)
    monkeypatch.setattr(
        asyncio,
        "run",
        lambda coro: asyncio.get_event_loop().run_until_complete(coro),
    )

    dummy_command = Command(name="dummy")
    ctx = Context(command=dummy_command)
    ctx.invoked_subcommand = None

    mod.main(ctx, quiet=False, verbose=False, fmt="human", pretty=True, debug=False)


def test_invoke_history_empty_quiet_skips_pretty(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Skip pretty print when quiet and empty history."""
    import typer.testing as typer_testing

    class _Runner:
        def invoke(
            self, _app: Any, tokens: list[str], env: dict[str, str] | None = None
        ) -> _FakeResult:  # noqa: ARG002
            return _FakeResult(0, '{"entries": []}\n', "")

    monkeypatch.setattr(typer_testing, "CliRunner", lambda: _Runner())
    out, err, mp = _capture_io()
    try:
        code = mod._invoke(  # pyright: ignore[reportPrivateUsage]
            ["history"], repl_quiet=True
        )
        assert code == 0
        assert out.getvalue() == ""
        assert err.getvalue() == ""
    finally:
        mp.undo()


def test_run_piped_edges_empty_segs_docs_quiet_unknown_and_empty_tokens(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Cover edge cases in piped processing."""
    import shlex

    monkeypatch.setattr(mod, "_known_commands", lambda: ["status", "config"])
    monkeypatch.setattr(mod, "_suggest", lambda s: None)
    real_split = shlex.split

    def split_or_empty(s: str) -> list[str]:
        if s == "EMPTYTOK":
            return []
        return real_split(s)

    monkeypatch.setattr(shlex, "split", split_or_empty)
    calls: list[list[str]] = []

    def fake_invoke(t: list[str], repl_quiet: bool) -> int:
        calls.append(t)
        return 0

    monkeypatch.setattr(mod, "_invoke", fake_invoke)

    _run_piped_lines(
        [
            "status;;",
            "docs",
            "docs topic",
            "EMPTYTOK",
            "config",
            "config get",
            "not_a_cmd",
        ],
        quiet=True,
    )

    out = capsys.readouterr()
    assert out.out == ""
    assert out.err == ""
    assert ["status"] in calls
    assert ["config"] in calls


def test_completer_group_list_both_paths_and_help_false(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Complete group list both branches and no help match."""
    comp = make_fake_completer()
    monkeypatch.setattr(typer, "Typer", (FakeApp,))
    assert "set" in complete_text(comp, "config s")
    assert complete_text(comp, "config x") == []
    assert "--help" not in complete_text(comp, "status --x")


class _KBRecorder:  # pyright: ignore[reportUnusedClass]
    """Fake keybindings that exercise branches."""

    def add(
        self, _key: str
    ) -> Callable[[Callable[[Any], None]], Callable[[Any], None]]:
        """Decorator that immediately calls handler."""

        def deco(fn: Callable[[Any], None]) -> Callable[[Any], None]:
            class _BufA:
                _started: bool
                _validated: bool
                complete_state: Any | None = None

                def start_completion(self, select_first: bool) -> None:  # noqa: ARG002
                    self._started = True

                def validate_and_handle(self) -> None:
                    self._validated = True

            class _State:
                pass

            class _BufB:  # pyright: ignore[reportUnusedClass]
                _next: bool
                _applied: bool
                complete_state: _State = _State()

                def complete_next(self) -> None:
                    self._next = True

                def apply_completion(self, _: Any) -> None:
                    self._applied = True

            class _App:
                current_buffer: object

            class _Evt:
                app: _App

            e1 = _Evt()
            e1.app = _App()
            e1.app.current_buffer = _BufA()
            fn(e1)
            return fn

        return deco


class PSValueErrorThenQuit:
    """Fake PromptSession that raises ValueError then quits."""

    def __init__(self, *a: Any, **k: Any) -> None:
        """Init."""
        self._i: int = 0

    async def prompt_async(self) -> str:
        """Yield lines causing ValueError then unknown then quit."""
        self._i += 1
        if self._i == 1:
            return '"'
        if self._i == 2:
            return "unknown"
        return "quit"


def _wire_interactive_env(monkeypatch: pytest.MonkeyPatch, app: Any) -> None:
    """Wire fake interactive environment and keybindings."""
    import importlib
    import subprocess as _subp

    from prompt_toolkit import key_binding as _kb_mod

    fake_cli_mod = types.SimpleNamespace(build_app=lambda: app)
    monkeypatch.setattr(importlib, "import_module", lambda _: fake_cli_mod)

    class _KeyBindings:
        last: _KeyBindings | None = None

        def __init__(self) -> None:
            type(self).last = self
            self.handlers: dict[str, Callable[[Any], None]] = {}

        def add(
            self, key: str
        ) -> Callable[[Callable[[Any], None]], Callable[[Any], None]]:
            def deco(fn: Callable[[Any], None]) -> Callable[[Any], None]:
                self.handlers[key] = fn
                return fn

            return deco

    monkeypatch.setattr(_kb_mod, "KeyBindings", _KeyBindings)
    monkeypatch.setattr(
        _subp, "run", lambda *a, **k: types.SimpleNamespace(returncode=0)
    )


@pytest.mark.asyncio
async def test_run_interactive_keybindings_and_value_error_and_unknown(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Handle ValueError in interactive and unknown command."""
    config_app = FakeApp([], [])
    config_group = FakeGroup("config", config_app)
    app = FakeApp([], [config_group])

    _wire_interactive_env(monkeypatch, app)
    import prompt_toolkit

    monkeypatch.setattr(prompt_toolkit, "PromptSession", PSValueErrorThenQuit)
    monkeypatch.setattr(mod, "_known_commands", lambda: ["config"])
    monkeypatch.setattr(mod, "_suggest", lambda s: None)

    await mod._run_interactive()  # pyright: ignore[reportPrivateUsage]

    out = capsys.readouterr()
    assert "No such command 'unknown'." in (out.err or out.out)


@pytest.mark.asyncio
async def test_run_interactive_exits_on_keyboard_interrupt(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Exit interactive on KeyboardInterrupt."""
    _wire_interactive_env(monkeypatch, FakeApp([], []))

    class PSKI:
        def __init__(self, *a: Any, **k: Any) -> None:
            """Init."""
            return None

        async def prompt_async(self) -> str:
            """Raise KeyboardInterrupt."""
            raise KeyboardInterrupt

    import prompt_toolkit

    monkeypatch.setattr(prompt_toolkit, "PromptSession", PSKI)
    await mod._run_interactive()  # pyright: ignore[reportPrivateUsage]
    out = capsys.readouterr()
    assert "Exiting REPL." in out.out


def test_main_guard_invokes_repl_app_without_side_effects(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Run module as __main__ without side effects."""
    import runpy
    import sys as _sys
    import types as _types

    class _DummyTyper:
        def __init__(self, *a: Any, **k: Any) -> None:
            """Init."""
            self.registered_commands: list[Any] = []
            self.registered_groups: list[Any] = []

        def __call__(self, *a: Any, **k: Any) -> None:
            """Call."""
            return None

        def callback(
            self, *a: Any, **k: Any
        ) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
            """Return identity decorator."""

            def deco(fn: Callable[..., Any]) -> Callable[..., Any]:
                return fn

            return deco

    fake_typer = _types.SimpleNamespace(
        Typer=_DummyTyper,
        Option=lambda default=False, *a, **k: default,
        testing=_types.SimpleNamespace(CliRunner=lambda: _RecordingFakeRunner()),
    )
    monkeypatch.setitem(_sys.modules, "typer", fake_typer)
    runpy.run_module("bijux_cli.commands.repl", run_name="__main__", alter_sys=True)


@pytest.mark.asyncio
async def test_run_interactive_keybindings_apply_completion(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Apply completion on enter keybinding."""
    app = FakeApp([], [])
    _wire_interactive_env(monkeypatch, app)

    class PSOnce:
        def __init__(self, *a: Any, **k: Any) -> None:
            """Init."""
            self._done = False

        async def prompt_async(self) -> str:
            """Yield once then EOF."""
            if not self._done:
                self._done = True
                return ""
            raise EOFError

    import prompt_toolkit

    monkeypatch.setattr(prompt_toolkit, "PromptSession", PSOnce)
    await mod._run_interactive()  # pyright: ignore[reportPrivateUsage]

    from prompt_toolkit.completion import Completion

    kb = getattr(
        prompt_toolkit.key_binding.KeyBindings,  # pyright: ignore[reportAttributeAccessIssue]
        "last",
        None,
    )
    assert kb is not None
    enter = kb.handlers["enter"]

    class _State:
        def __init__(self) -> None:
            """Init."""
            self.current_completion = Completion("x", 0)

    class _Buf:
        def __init__(self) -> None:
            """Init."""
            self.complete_state = _State()
            self.applied: Any | None = None

        def apply_completion(self, c: Any) -> None:
            """Record completion applied."""
            self.applied = c

    class _App:
        def __init__(self) -> None:
            """Init."""
            self.current_buffer = _Buf()

    class _Evt:
        def __init__(self) -> None:
            """Init."""
            self.app = _App()

    evt = _Evt()
    enter(evt)
    assert evt.app.current_buffer.applied is not None


def test_run_piped_config_get_missing_arg_emits_json(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Emit JSON error for missing config get argument."""
    monkeypatch.setattr(mod, "_known_commands", lambda: ["config"])
    old_stdin, sys.stdin = sys.stdin, io.StringIO("config get\n")
    out, err, mp = _capture_io()
    try:
        with pytest.raises(SystemExit) as se:
            mod._run_piped(repl_quiet=False)  # pyright: ignore[reportPrivateUsage]
        assert se.value.code == 0
    finally:
        mp.undo()
        sys.stdin = old_stdin
    payload = json.loads(out.getvalue().strip())
    assert payload["failure"] == "missing_argument"
    assert payload["command"] == "config get"


def test_completer_force_key_value_and_dummy(monkeypatch: pytest.MonkeyPatch) -> None:
    """Offer KEY=VALUE and non-empty root suggestions."""
    comp = make_fake_completer()
    monkeypatch.setattr(typer, "Typer", (FakeApp,))
    suggestions = complete_text(comp, "config set ")
    assert {"KEY=VALUE", "--help"} & set(suggestions)
    assert complete_text(comp, "") != []


def test_run_piped_skips_empty_and_comment_segments(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Treat leading semicolon as unknown command."""
    monkeypatch.setattr(mod, "_known_commands", lambda: ["config"])
    old_stdin, sys.stdin = sys.stdin, io.StringIO(" ;   #comment only\n")
    out, err, mp = _capture_io()
    try:
        with pytest.raises(SystemExit):
            mod._run_piped(repl_quiet=False)  # pyright: ignore[reportPrivateUsage]
    finally:
        mp.undo()
        sys.stdin = old_stdin
    e = err.getvalue()
    assert "No such command" in e
    assert ";   #comment only" in e


def test_run_piped_prints_prompt_for_pure_blank_or_comment(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Print prompt for pure blank or comment lines."""
    monkeypatch.setattr(mod, "_known_commands", lambda: ["config"])
    old_stdin, sys.stdin = sys.stdin, io.StringIO("\n# only comment\n")
    out, err, mp = _capture_io()
    try:
        with pytest.raises(SystemExit):
            mod._run_piped(repl_quiet=False)  # pyright: ignore[reportPrivateUsage]
    finally:
        mp.undo()
        sys.stdin = old_stdin
    s = err.getvalue()
    assert "bijux>" in s
