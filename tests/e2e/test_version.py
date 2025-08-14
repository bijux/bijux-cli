# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux version` command."""

from __future__ import annotations

import json
import re
import signal
import string
from subprocess import PIPE, CompletedProcess, Popen
import sys
import threading
import time
from typing import Any

from _pytest.monkeypatch import MonkeyPatch
from hypothesis import HealthCheck, given, settings, strategies
from hypothesis.strategies import DrawFn, composite, data, lists, sampled_from, text
import pytest
import yaml

from bijux_cli.__version__ import __version__ as bijux_version
from tests.e2e.conftest import run_cli

ALPHABET = string.ascii_letters + string.digits + "_-."
ALL_FLAGS = [
    "--help",
    "-h",
    "--quiet",
    "-q",
    "--verbose",
    "-v",
    "--debug",
    "-d",
    "--no-pretty",
    "--format",
    "-f",
]
FORMATS = ["json", "yaml", "bogus", "123", ""]
SEMVER = re.compile(r"\d+\.\d+\.\d+")


def _no_stacktrace_leak(text: str) -> None:
    """Assert no traceback or framework names leak into user output."""
    s = text.lower()
    assert "traceback" not in s
    assert "typer" not in s
    assert "click" not in s


def flags_verbose(flags: list[str]) -> bool:
    """Check if verbose flags are present."""
    return "--verbose" in flags or "-v" in flags


def flags_debug(flags: list[str]) -> bool:
    """Check if debug flags are present."""
    return "--debug" in flags or "-d" in flags


@composite
def flag_permutations(draw: DrawFn) -> list[str]:
    """Generate permutations of command-line flags."""
    base = draw(lists(sampled_from(ALL_FLAGS), min_size=0, max_size=5, unique=False))
    assert isinstance(base, list)
    if any(f in base for f in ("--format", "-f")):
        fmt_flag = draw(sampled_from(["--format", "-f"]))
        fmt_value = draw(sampled_from(FORMATS))
        return [f for f in base if f not in ("--format", "-f")] + [fmt_flag, fmt_value]
    return base


def assert_version_output(
    data: dict[str, Any], *, verbose: bool = False, debug: bool = False
) -> None:
    """Assert the structure and content of a successful version command output."""
    assert "version" in data
    assert data["version"] == bijux_version
    if verbose or debug:
        assert "python" in data
        assert "platform" in data
        assert "timestamp" in data
        assert isinstance(data["timestamp"], float)
    else:
        assert "timestamp" not in data


def assert_error_output(data: dict[str, Any]) -> None:
    """Assert the structure of an error output."""
    assert "error" in data
    assert isinstance(data["error"], str)


def parse_output(fmt: str, out: str) -> dict[str, Any]:
    """Parse the output string based on the specified format."""
    data: Any
    if fmt == "json":
        data = json.loads(out)
    elif fmt == "yaml":
        data = yaml.safe_load(out)
    else:
        raise AssertionError(f"Unknown format: {fmt}")
    if not isinstance(data, dict):
        raise TypeError(
            f"Expected dict output for {fmt}, got {type(data).__name__}: {data!r}"
        )
    return data


@pytest.mark.parametrize(
    ("flags", "fmt", "verbose", "debug"),
    [
        ([], "json", False, False),
        (["--format", "json"], "json", False, False),
        (["-f", "yaml"], "yaml", False, False),
        (["-v"], "json", True, False),
        (["--verbose", "--format", "yaml"], "yaml", True, False),
        (["--debug"], "json", False, True),
        (["--debug", "--format", "yaml"], "yaml", False, True),
        (["--debug", "--no-pretty"], "json", False, True),
    ],
)
def test_version_contract(
    flags: list[str], fmt: str, verbose: bool, debug: bool
) -> None:
    """Test the version command with various flag combinations."""
    res: CompletedProcess[str] = run_cli(["version"] + flags)
    assert res.returncode == 0
    data = parse_output(fmt, res.stdout)
    assert_version_output(data, verbose=verbose, debug=debug)
    assert "traceback" not in (res.stdout + res.stderr).lower()
    assert "warning" not in (res.stdout + res.stderr).lower()


@pytest.mark.parametrize("flag", ["--help", "-h"])
def test_version_help_output(flag: str) -> None:
    """Test the help output for the version command."""
    res = run_cli(["version", flag])
    assert res.returncode == 0
    out = res.stdout
    assert out.startswith("Usage:")
    assert "Options:" in out
    for k in ("--help", "-h", "--quiet", "-q", "--verbose", "-v"):
        assert k in out


@pytest.mark.parametrize("flag", ["--quiet", "-q"])
def test_version_quiet(flag: str) -> None:
    """Test the quiet flag for the version command."""
    res = run_cli(["version", flag])
    assert res.returncode == 0
    assert not res.stdout.strip()
    assert not res.stderr.strip()


@pytest.mark.parametrize(
    ("flags", "expect"),
    [
        (["--quiet", "--verbose"], False),
        (["-q", "-v"], False),
        (["--format", "yaml", "-q"], False),
        (["--verbose", "-f", "yaml"], True),
        (["--debug", "--no-pretty"], True),
    ],
)
def test_version_flag_output_precedence(flags: list[str], expect: bool) -> None:
    """Test the precedence of flags affecting output verbosity."""
    res = run_cli(["version"] + flags)
    assert bool(res.stdout.strip()) is expect


@pytest.mark.parametrize(
    ("flags", "expect"),
    [
        (["--format", "json", "--format", "yaml"], "yaml"),
        (["-f", "yaml", "-f", "json"], "json"),
    ],
)
def test_version_duplicate_flag_last_win(flags: list[str], expect: str) -> None:
    """Test that the last of duplicate flags takes precedence."""
    res = run_cli(["version"] + flags)
    out = json.loads(res.stdout) if expect == "json" else yaml.safe_load(res.stdout)
    assert "version" in out


@pytest.mark.parametrize("fmt", ["json", "yaml"])
def test_version_no_pretty_flag(fmt: str) -> None:
    """Test the --no-pretty flag suppresses formatted output."""
    res = run_cli(["version", "--format", fmt, "--no-pretty"])
    assert res.returncode == 0
    assert res.stdout.count("\n") <= 1


@pytest.mark.parametrize("flag", ["--debug", "-d"])
def test_version_debug_output(flag: str) -> None:
    """Test the debug flag produces debug information."""
    res = run_cli(["version", flag])
    assert res.returncode == 0
    out = json.loads(res.stdout)
    assert_version_output(out, debug=True)
    assert res.stderr.strip()


@pytest.mark.parametrize("flag", ["--debug", "-d"])
def test_version_debug_pretty_overrides_no_pretty(flag: str) -> None:
    """Test that debug flag's pretty printing overrides --no-pretty."""
    res = run_cli(["version", flag, "--no-pretty"])
    assert res.returncode == 0
    assert res.stdout.count("\n") >= 2


@pytest.mark.parametrize(
    ("flag", "value"),
    [
        ("--format", "xml"),
        ("-f", ""),
        ("--format", "123"),
    ],
)
def test_version_invalid_format(flag: str, value: str) -> None:
    """Test that invalid format values produce an error."""
    res = run_cli(["version", flag, value])
    assert res.returncode != 0
    data = json.loads(res.stdout or res.stderr)
    assert_error_output(data)


@pytest.mark.parametrize(
    ("env_var", "value", "expect_code"),
    [
        ("BIJUXCLI_VERSION", "v1.0.β", 3),
        ("BIJUXCLI_VERSION", "v" + "9" * 10000, 2),
        ("BIJUXCLI_VERSION", "1.0.\udc80", 2),
    ],
)
def test_version_env_var_non_ascii(
    monkeypatch: Any, env_var: str, value: str, expect_code: int
) -> None:
    """Test handling of non-ASCII environment variable values."""
    monkeypatch.setenv(env_var, value)
    res = run_cli(["version"])
    assert res.returncode == expect_code or res.returncode != 0
    data = json.loads(res.stdout or res.stderr)
    assert_error_output(data)


def test_version_env_var_case_sensitive(monkeypatch: Any) -> None:
    """Test that environment variables are case-sensitive."""
    monkeypatch.setenv("bijuxcli_version", "bogus")
    res = run_cli(["version"])
    assert res.returncode == 0
    out = json.loads(res.stdout)
    assert out["version"] == bijux_version


def test_version_idempotency_and_no_config_pollution(
    monkeypatch: Any, tmp_path: Any
) -> None:
    """Runs are idempotent; user config, LANG, or history do not leak."""
    config = tmp_path / "config.toml"
    config.write_text("bad = 'pollute'")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(config))
    monkeypatch.setenv("LANG", "C")
    runs = [json.loads(run_cli(["version"]).stdout) for _ in range(2)]
    for r in runs:
        assert_version_output(r)
        assert "bad" not in r
    assert runs[0] == runs[1]


def test_version_parallel_and_perf() -> None:
    """Parallel and fast."""
    results: list[CompletedProcess[str]] = []

    def f() -> None:
        """Executes the 'version' command and appends the result to a list."""
        results.append(run_cli(["version"]))

    threads = [threading.Thread(target=f) for _ in range(4)]
    start = time.perf_counter()
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    elapsed = time.perf_counter() - start
    for r in results:
        assert r.returncode == 0
        assert "version" in json.loads(r.stdout)
    assert elapsed < 5.0, f"Parallel CLI slow: {elapsed}s"


def test_version_verbose_timestamp() -> None:
    """Test that verbose output includes a timestamp."""
    out = json.loads(run_cli(["version", "-v"]).stdout)
    assert isinstance(out.get("timestamp"), float)


def test_version_default_has_no_timestamp() -> None:
    """Test that default output does not include a timestamp."""
    out = json.loads(run_cli(["version"]).stdout)
    assert "timestamp" not in out


@settings(max_examples=10, deadline=None, suppress_health_check=[HealthCheck.too_slow])
@given(data=data())
def test_version_hypothesis_flags(data: Any) -> None:
    """Test various flag combinations using hypothesis."""
    flags: list[str] = data.draw(flag_permutations())
    res = run_cli(["version"] + flags)

    if "--help" in flags or "-h" in flags:
        assert res.returncode == 0
        assert res.stdout.startswith("Usage:")
        assert "Options:" in res.stdout
        return
    if "--quiet" in flags or "-q" in flags:
        assert not res.stdout
        invalid_format = False
        for i, flag in enumerate(flags):
            if flag in ("--format", "-f") and i + 1 < len(flags):
                val = flags[i + 1].lower()
                if val not in ("json", "yaml"):
                    invalid_format = True
        if invalid_format:
            assert res.returncode != 0
        else:
            assert res.returncode == 0
        return

    fmt = "json"
    for i, flag in enumerate(flags):
        if (
            flag in ("--format", "-f")
            and i + 1 < len(flags)
            and flags[i + 1].lower() in ("json", "yaml")
        ):
            fmt = flags[i + 1].lower()

    try:
        out = parse_output(fmt, res.stdout)
        assert_version_output(
            out, verbose=flags_verbose(flags), debug=flags_debug(flags)
        )
    except Exception:
        assert res.returncode != 0


@settings(max_examples=10, deadline=None)
@given(
    fmt=(
        text(alphabet=ALPHABET, min_size=1, max_size=8).filter(
            lambda s: not s.startswith("-")
        )
    )
)
def test_version_fuzz_format(fmt: str) -> None:
    """Test various format strings using hypothesis."""
    res = run_cli(["version", "--format", fmt])
    if fmt.lower() in ("json", "yaml"):
        out = parse_output(fmt.lower(), res.stdout)
        assert_version_output(out)
    else:
        assert res.returncode != 0
        data = json.loads(res.stdout or res.stderr)
        assert_error_output(data)


@pytest.mark.parametrize("flag", ["--help", "-h"])
def test_version_help_contract(flag: str) -> None:
    """Test help output contract: human-readable, no leaks."""
    res = run_cli(["version", flag])
    assert res.returncode == 0
    out = res.stdout.lower()
    assert "usage:" in out
    assert "options:" in out
    _no_stacktrace_leak(out)


def test_version_signal_interrupt() -> None:
    """Test SIGINT during version (quick, but ensure clean exit)."""
    proc = Popen(  # noqa: S603
        [sys.executable, "-m", "bijux_cli", "version"],
        stdout=PIPE,
        stderr=PIPE,
        text=True,
    )
    time.sleep(0.1)
    proc.send_signal(signal.SIGINT)
    stdout, stderr = proc.communicate(timeout=2)
    assert proc.returncode != 0
    msg = (stdout + stderr).lower()
    assert "interrupt" in msg or "signal" in msg


@given(
    env_val=strategies.text(
        min_size=1,
        max_size=10,
        alphabet=strategies.characters(
            blacklist_categories=[
                "C",
                "Z",
                "M",
                "N",
            ],
            blacklist_characters=["\x00"],
        ),
    )
)
@settings(
    max_examples=10,
    deadline=None,
    suppress_health_check=[HealthCheck.function_scoped_fixture],
)
def test_version_fuzz_env_vars(env_val: str, monkeypatch: MonkeyPatch) -> None:
    """Fuzz BIJUXCLI_VERSION for leaks/non-ASCII errors."""
    monkeypatch.setenv("BIJUXCLI_VERSION", env_val)
    res = run_cli(["version"])
    if any(ord(c) > 127 for c in env_val) or not SEMVER.fullmatch(env_val):
        assert res.returncode == 3
    else:
        assert res.returncode == 0
        data = json.loads(res.stdout)
        assert_version_output(data)


def normalize_version_output(data: dict[str, Any]) -> dict[str, Any]:
    """Remove fields known to be volatile between environments."""
    data = dict(data)
    data.pop("python", None)
    data.pop("platform", None)
    data.pop("timestamp", None)
    return data
