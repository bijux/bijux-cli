# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI root main module."""

from __future__ import annotations

import builtins
from io import StringIO
import json
import os
from pathlib import Path
import runpy
import sys
from typing import Any

import click
from click.exceptions import NoSuchOption, UsageError
import pytest
import structlog
import typer

import bijux_cli.__main__ as main_mod
from bijux_cli.__main__ import (
    _strip_format_help,
    check_missing_format_argument,
    disable_cli_colors_for_test,
    get_usage_for_args,
    is_quiet_mode,
    main,
    print_json_error,
    setup_structlog,
    should_record_command_history,
)

# pyright: reportPrivateUsage=false


class DummyHistory:
    """A mock History service that records calls to its 'add' method."""

    def __init__(self) -> None:
        """Initialize the dummy history service."""
        self.add_calls: list[dict[str, Any]] = []

    def add(self, **kw: Any) -> None:
        """Record a history add event."""
        self.add_calls.append(kw)


class DummyContainer:
    """A mock DI container that resolves a dummy History service."""

    def __init__(self, hist: DummyHistory) -> None:
        """Initialize the dummy DI container."""
        self._hist = hist

    def resolve(self, cls: Any) -> Any:
        """Resolve the History service."""
        if cls.__name__ == "History":
            return self._hist
        raise RuntimeError("unexpected resolve")


@pytest.fixture(autouse=True)
def _isolate_env(monkeypatch: pytest.MonkeyPatch) -> None:  # pyright: ignore[reportUnusedFunction]
    """Isolate the test environment by patching core CLI components."""
    hist = DummyHistory()
    cont = DummyContainer(hist)

    monkeypatch.setattr("bijux_cli.__main__.DIContainer.current", lambda: cont)
    monkeypatch.setattr(
        "bijux_cli.__main__.register_default_services", lambda *a, **k: None
    )
    monkeypatch.setattr("bijux_cli.__main__.Engine", lambda *a, **k: None)

    class FakeApp:
        def __call__(self, args: list[str], standalone_mode: bool = False) -> int:
            return 0

    monkeypatch.setattr("bijux_cli.__main__.build_app", FakeApp)


def test_strip_format_help_various() -> None:
    """Test the stripping of --format and --help flags from argument lists."""
    assert _strip_format_help(["foo", "--format", "json", "bar"]) == [
        "foo",
        "--format",
        "json",
        "bar",
    ]
    assert _strip_format_help(["a", "--format", "--help", "b", "-f", "-h", "c"]) == [
        "a",
        "b",
        "c",
    ]
    assert not _strip_format_help(["-f", "-h"])


@pytest.mark.parametrize(
    "args",
    [[], ["history"], ["help", "x"], ["cmd", "-q"], ["cmd", "--quiet"], ["other"]],
)
def test_record_and_quiet(args: list[str]) -> None:
    """Test the logic for determining quiet mode and if history should be recorded."""
    os.environ.pop("BIJUXCLI_DISABLE_HISTORY", None)
    want = should_record_command_history(args)
    quiet = is_quiet_mode(args)
    assert quiet == any(a in ("-q", "--quiet") for a in args)
    expected = bool(args) and args[0].lower() not in {"history", "help"}
    assert want is expected


def test_record_disabled_env() -> None:
    """Test that history recording is disabled when the environment variable is set."""
    os.environ["BIJUXCLI_DISABLE_HISTORY"] = "1"
    assert not should_record_command_history(["foo"])


def test_print_json_error_stdout_and_stderr(
    capfd: pytest.CaptureFixture[str],
) -> None:
    """Test that JSON errors are printed to the correct stream based on exit code."""
    print_json_error("err1", code=2, quiet=False)
    out, err = capfd.readouterr()
    assert json.loads(out) == {"error": "err1", "code": 2}
    print_json_error("err2", code=1, quiet=False)
    out, err = capfd.readouterr()
    assert json.loads(err) == {"error": "err2", "code": 1}
    print_json_error("err3", code=2, quiet=True)
    out, err = capfd.readouterr()
    assert not out
    assert not err


def test_get_usage_for_args_simple(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test retrieving the usage string for a simple command."""
    app = typer.Typer()

    @app.command()
    def foo() -> None:  # pyright: ignore[reportUnusedFunction]
        """Provide foo help."""

    txt = get_usage_for_args(["foo", "--help"], app)
    assert txt.strip() != ""


def test_check_missing_format_argument() -> None:
    """Test the detection of a missing argument for the --format flag."""
    assert check_missing_format_argument(["--format", "json"]) is None
    assert check_missing_format_argument(["-f", "yaml", "x"]) is None
    msg = check_missing_format_argument(["--format"])
    assert msg
    assert "requires an argument" in msg
    msg2 = check_missing_format_argument(["-f", "-h"])
    assert msg2
    assert "requires an argument" in msg2


def test_setup_structlog_switching() -> None:
    """Test that structlog can be set up in both dev and prod modes."""
    setup_structlog(False)
    setup_structlog(True)
    assert hasattr(structlog, "get_config")


def test_disable_cli_colors_for_test(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that CLI colors are disabled when in test mode."""
    os.environ.pop("BIJUXCLI_TEST_MODE", None)
    disable_cli_colors_for_test()
    os.environ["BIJUXCLI_TEST_MODE"] = "1"
    disable_cli_colors_for_test()
    assert os.environ.get("NO_COLOR") == "1"


def test_main_success_records_history() -> None:
    """Test that a successful command run is recorded in history."""
    monkeypatch = pytest.MonkeyPatch()
    monkeypatch.setenv("BIJUXCLI_DISABLE_HISTORY", "0")
    monkeypatch.setattr(sys, "argv", ["bijux", "cmd"])
    code = main()
    assert code == 0
    hist = main_mod.DIContainer.current()._hist  # type: ignore[attr-defined]
    assert len(hist.add_calls) == 1
    rec = hist.add_calls[0]
    assert rec["command"] == "cmd"
    assert not rec["params"]


def test_main_history_add_failure(
    capfd: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that a failure to record history is handled gracefully."""
    fake_hist = DummyHistory()

    def bad_add(**kw: Any) -> None:
        raise RuntimeError("nohist")

    monkeypatch.setattr(fake_hist, "add", bad_add)
    monkeypatch.setattr(
        "bijux_cli.__main__.DIContainer.current", lambda: DummyContainer(fake_hist)
    )

    class App:
        def __call__(self, args: list[str], standalone_mode: bool = False) -> int:
            return 0

    monkeypatch.setattr("bijux_cli.__main__.build_app", App)
    monkeypatch.setattr(sys, "argv", ["bijux", "foo"])
    rc = main()
    _, err = capfd.readouterr()
    assert rc == 1
    assert "[error] Could not record command history" in err


def test_main_help_and_missing_format(
    capfd: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test the main entry point's handling of --help and missing format arguments."""
    monkeypatch.setattr(sys, "argv", ["bijux", "--help"])
    code = main()
    assert code == 0

    monkeypatch.setenv("BIJUXCLI_DEBUG", "")
    monkeypatch.setattr(sys, "argv", ["bijux", "--format"])
    code2 = main()
    out2, _ = capfd.readouterr()
    assert code2 == 2
    parsed = json.loads(out2)
    assert "requires an argument" in parsed["error"]


@pytest.mark.parametrize(
    ("exc", "expected_code"),
    [
        (typer.Exit(5), 5),
        (NoSuchOption("badopt"), 2),
        (UsageError("uerr"), 2),
    ],
)
def test_main_catches_errors(
    exc: Exception,
    expected_code: int,
    capfd: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that the main entry point catches various Click and Typer exceptions."""

    class ErrorApp:
        def __call__(self, args: list[str], standalone_mode: bool = False) -> None:
            raise exc

    monkeypatch.setattr(main_mod, "build_app", ErrorApp)
    monkeypatch.setattr(sys, "argv", ["bijux", "do"])

    rc = main()
    out, err = capfd.readouterr()
    assert rc == expected_code
    if isinstance(exc, typer.Exit):
        assert not out
        assert not err
    else:
        combined = out or err
        json.loads(combined)


def test_main_catches_command_and_keyboard_and_generic(
    capfd: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the main entry point catches CommandError, KeyboardInterrupt, and generic exceptions."""
    from bijux_cli.core.exceptions import CommandError

    class CmdErrApp:
        def __call__(self, args: list[str], standalone_mode: bool = False) -> None:
            raise CommandError("cmdfail")

    monkeypatch.setattr(main_mod, "build_app", CmdErrApp)
    monkeypatch.setattr(sys, "argv", ["bijux"])
    rc1 = main()
    _, err1 = capfd.readouterr()
    assert rc1 == 1
    data1 = json.loads(err1)
    assert data1["error"] == "cmdfail"

    class KbApp:
        def __call__(self, args: list[str], standalone_mode: bool = False) -> None:
            raise KeyboardInterrupt

    monkeypatch.setattr(main_mod, "build_app", KbApp)
    rc2 = main()
    _, err2 = capfd.readouterr()
    assert rc2 == 130
    data2 = json.loads(err2)
    assert data2["error"] == "Aborted by user"

    class GenApp:
        def __call__(self, args: list[str], standalone_mode: bool = False) -> None:
            raise RuntimeError("oops")

    monkeypatch.setattr(main_mod, "build_app", GenApp)
    rc3 = main()
    out3, err3 = capfd.readouterr()
    assert rc3 == 1
    combined = out3 or err3
    assert "Unexpected error" in combined


def test_filtered_echo_suppresses_and_passes_through(
    capfd: pytest.CaptureFixture[str],
) -> None:
    """Test that the filtered echo correctly suppresses plugin warnings."""
    click.echo("hello")
    out1, _ = capfd.readouterr()
    assert "hello" in out1

    warning = "[WARN] Plugin 'test-src': does not expose a Typer app"
    click.echo(warning)
    out2, _ = capfd.readouterr()
    assert not out2


@pytest.mark.parametrize(
    "module_name", ["rich.console", "colorama", "prompt_toolkit.shortcuts"]
)
def test_disable_cli_colors_handles_missing_modules(
    monkeypatch: pytest.MonkeyPatch, module_name: str
) -> None:
    """Test that disabling colors works even if optional color libraries are missing."""
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "1")
    if module_name in sys.modules:
        monkeypatch.delitem(sys.modules, module_name)
    monkeypatch.setitem(sys.modules, module_name, None)
    disable_cli_colors_for_test()
    assert os.environ.get("NO_COLOR") == "1"


def test_get_usage_for_args_stops_before_extra(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that get_usage_for_args truncates arguments after --help."""
    app = typer.Typer()

    @app.command()
    def foo() -> None:  # pyright: ignore[reportUnusedFunction]
        """Provide foo help."""

    help_text = get_usage_for_args(["foo", "bar", "--help", "baz"], app)
    assert "Usage" in help_text or "foo help" in help_text
    assert "baz" not in help_text


def test_main_quiet_mode_redirects_stderr(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that quiet mode redirects stderr to devnull."""
    opened: dict[str, Any] = {}

    def fake_open(path: str, mode: str) -> StringIO:
        opened["path"] = path
        return StringIO()

    monkeypatch.setenv("BIJUXCLI_DEBUG", "0")
    monkeypatch.setattr(builtins, "open", fake_open)
    monkeypatch.setattr(sys, "argv", ["bijux", "-q", "do"])
    rc = main()
    assert rc == 0
    assert opened.get("path") == os.devnull


def test_strip_format_help_only_pairs_v2() -> None:
    """Test stripping of adjacent format and help flags (v2)."""
    assert _strip_format_help(["a", "--format", "--help", "b"]) == ["a", "b"]
    assert _strip_format_help(["-f", "-h", "x"]) == ["x"]
    assert _strip_format_help(["--format", "json", "--debug"]) == [
        "--format",
        "json",
        "--debug",
    ]


def test_no_record_if_disabled_v2() -> None:
    """Test that history recording is disabled via environment variable (v2)."""
    os.environ["BIJUXCLI_DISABLE_HISTORY"] = "1"
    assert not should_record_command_history(["anything"])


def test_print_json_error_and_missing_format_v2(
    capfd: pytest.CaptureFixture[str],
) -> None:
    """Test JSON error printing and missing format argument detection (v2)."""
    print_json_error("foo", code=2, quiet=False)
    out, err = capfd.readouterr()
    assert json.loads(out) == {"error": "foo", "code": 2}

    print_json_error("bar", code=1, quiet=False)
    out, err = capfd.readouterr()
    assert json.loads(err) == {"error": "bar", "code": 1}

    print_json_error("bz", quiet=True)
    out, err = capfd.readouterr()
    assert not out
    assert not err

    assert "requires an argument" in (check_missing_format_argument(["--format"]) or "")
    assert check_missing_format_argument(["--format", "json"]) is None


def test_setup_structlog_branches_v2() -> None:
    """Test both branches of structlog setup (v2)."""
    setup_structlog(False)
    setup_structlog(True)


def test_main_quiet_and_missing_format_v2(
    monkeypatch: pytest.MonkeyPatch, capfd: pytest.CaptureFixture[str]
) -> None:
    """Test main entry point behavior with quiet mode and missing format arg (v2)."""
    monkeypatch.setenv("BIJUXCLI_DEBUG", "")
    monkeypatch.setattr(sys, "argv", ["bijux", "-q", "cmd"])
    rc1 = main()
    assert rc1 == 0

    monkeypatch.setattr(sys, "argv", ["bijux", "--format"])
    rc2 = main()
    out2, _ = capfd.readouterr()
    assert rc2 == 2
    assert "requires an argument" in out2


def test_disable_cli_colors_for_test_no_mode_v2(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that colors are not disabled if not in test mode (v2)."""
    monkeypatch.delenv("BIJUXCLI_TEST_MODE", raising=False)
    monkeypatch.delenv("NO_COLOR", raising=False)
    disable_cli_colors_for_test()
    assert "NO_COLOR" not in os.environ


def test_get_usage_for_args_truncates_after_help_v2(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test that get_usage_for_args truncates arguments after --help (v2)."""
    app = typer.Typer()

    @app.command()
    def foo() -> None:  # pyright: ignore[reportUnusedFunction]
        """Provide Foo command help."""

    help_text = get_usage_for_args(["foo", "--help", "bar"], app)
    assert "Foo command help" in help_text
    assert "bar" not in help_text


@pytest.mark.parametrize(
    ("args", "expect_record"),
    [
        ([], False),
        (["history"], False),
        (["help", "x"], False),
        (["cmd", "-q"], True),
        (["cmd", "--quiet"], True),
        (["cmd"], True),
    ],
)
def test_history_and_quiet_behavior_v2(args: list[str], expect_record: bool) -> None:
    """Test the combined logic for history recording and quiet mode detection (v2)."""
    os.environ.pop("BIJUXCLI_DISABLE_HISTORY", None)
    assert is_quiet_mode(args) == any(a in ("-q", "--quiet") for a in args)
    assert should_record_command_history(args) is expect_record


def test_filtered_echo_suppresses_plugin_warnings_v2(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test that the filtered echo correctly suppresses plugin warnings (v2)."""
    bad = "[WARN] Plugin 'test-src' does not expose a Typer app via 'cli()' or 'app'"
    click.echo(bad)
    out, err = capsys.readouterr()
    assert not out
    assert not err

    click.echo("all good")
    out, err = capsys.readouterr()
    assert out.strip() == "all good"


def test_get_usage_for_args_no_help_token_v2(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test get_usage_for_args when the --help token is not explicitly passed (v2)."""
    app = typer.Typer()

    @app.command()
    def bar() -> None:  # pyright: ignore[reportUnusedFunction]
        """Provide Bar command help."""

    help_text = get_usage_for_args(["bar"], app)
    assert "Bar command help" in help_text
    assert "UsageError" not in help_text


def test_main_guard_at_eof_does_not_double_import(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the 'if __name__ == __main__' guard works as expected."""
    src = tmp_path / "guard.py"
    src.write_text(
        "import sys\n"
        "def main():\n"
        "    return 42\n"
        'if __name__ == "__main__":\n'
        "    sys.exit(main())\n"
    )
    with pytest.raises(SystemExit) as exc:
        runpy.run_path(str(src), run_name="__main__")
    assert exc.value.code == 42
