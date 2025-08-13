# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux status` command."""

from __future__ import annotations

from collections.abc import Callable
import json
from pathlib import Path
import random
import re
import string
import subprocess
import sys
import threading
import time
from typing import Any, cast

from hypothesis import HealthCheck, given, settings
import hypothesis.strategies as st
import psutil
import pytest
import yaml

from tests.e2e.conftest import run_cli

SUPPORTED_FORMATS = ["json", "yaml"]
FORMATS = ["json", "yaml", "bogus", "garbage", ""]

VALID_FLAGS = {
    "help": ["--help", "-h"],
    "quiet": ["--quiet", "-q"],
    "verbose": ["--verbose", "-v"],
    "debug": ["--debug", "-d"],
}
ALL_FLAGS = [
    "--help",
    "-h",
    "--quiet",
    "-q",
    "--verbose",
    "-v",
    "--debug",
    "-d",
    "--format",
    "-f",
    "--no-pretty",
]
ERROR_TERMS = ["error", "not supported", "invalid", "ascii", "non-ascii"]


def run_cli_watch(
    args: list[str], timeout: float = 1.0
) -> tuple[subprocess.Popen[str], list[str], str]:
    """Spawn the CLI process for watch tests, return process and output lines."""
    import os
    import signal

    proc = subprocess.Popen(  # noqa: S603
        [sys.executable, "-m", "bijux_cli", "status", *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env={**os.environ, "PYTHONUNBUFFERED": "1"},
    )
    lines: list[str] = []
    start = time.monotonic()
    try:
        while True:
            if proc.stdout is None:
                break
            line = proc.stdout.readline()
            if line:
                lines.append(line)
                if len(lines) >= 2:
                    break
            if time.monotonic() - start > timeout:
                break
    finally:
        proc.send_signal(signal.SIGINT)
        proc.wait(timeout=2)
    return proc, lines, proc.stdout.read() if proc.stdout else ""


def _flag_value_id(x: Any) -> str:
    """
    Return a unique test ID for parameterized flags.

    - For (list | tuple) of at least 2 elements, format as "flag=value" or "flag=empty".
    - Otherwise, just str(x).
    """
    if isinstance(x, list | tuple) and len(x) >= 2:
        flag, value = x[:2]
        return f"{flag}={value or 'empty'}"
    return str(x)


def load_output(stdout: str, fmt: str) -> dict[str, Any]:
    """Load CLI output into a dictionary based on the format."""
    loader = json.loads if fmt.lower() == "json" else yaml.safe_load
    data = loader(stdout)
    assert isinstance(data, dict), f"Output is not dict: {data!r}"
    return data


def assert_status_schema(
    data: dict[str, Any],
    *,
    expect_verbose: bool = False,
    expect_ts: bool = False,
    expect_watch_stopped: bool = False,
) -> None:
    """Assert the status output conforms to the expected schema."""
    if expect_watch_stopped:
        assert data.get("status") == "watch-stopped"
        return
    assert data.get("status") == "ok"
    if expect_ts:
        assert isinstance(data.get("ts"), float)
    if expect_verbose:
        assert isinstance(data.get("python"), str)
        assert isinstance(data.get("platform"), str)


@pytest.mark.parametrize(
    ("flag", "value", "loader"),
    [
        ("--format", "json", json.loads),
        ("-f", "json", json.loads),
        ("--format", "yaml", yaml.safe_load),
        ("-f", "yaml", yaml.safe_load),
        ("--format", "JSON", json.loads),
        ("--format", "YAML", yaml.safe_load),
    ],
    ids=_flag_value_id,
)
def test_status_format_flag(
    flag: str, value: str, loader: Callable[[str], Any]
) -> None:
    """Test the --format flag with various values."""
    res = run_cli(["status", flag, value])
    assert res.returncode == 0, f"Failed: {res.stderr}"
    data = loader(res.stdout)
    assert "status" in data
    assert data["status"] == "ok"


@pytest.mark.parametrize("flag", VALID_FLAGS["quiet"])
def test_status_quiet_flag(flag: str) -> None:
    """Test that the --quiet flag suppresses all output."""
    res = run_cli(["status", flag])
    assert res.returncode == 0
    assert not res.stdout.strip()
    assert not res.stderr.strip()


@pytest.mark.parametrize(
    "flags",
    [
        ["--quiet"],
        ["-q"],
        ["--quiet", "--format", "json"],
        ["-q", "--format", "yaml"],
        ["--quiet", "--format", "bogus"],
        ["--quiet", "--notarealoption"],
        ["-q", "-f", ""],
    ],
)
def test_status_quiet_contract(flags: list[str]) -> None:
    """Quiet mode: success (0) if valid, nonzero if invalid, always no output."""
    res = run_cli(["status", *flags])
    assert not res.stdout.strip()
    assert not res.stderr.strip()
    invalid_format = False
    for i, flag in enumerate(flags):
        if flag in ("--format", "-f") and i + 1 < len(flags):
            val = flags[i + 1].lower()
            if val not in ("json", "yaml"):
                invalid_format = True
    if any(f in flags for f in ["--notarealoption"]) or invalid_format:
        assert res.returncode != 0
    else:
        assert res.returncode == 0


@pytest.mark.parametrize("flag", VALID_FLAGS["verbose"])
@pytest.mark.parametrize("fmt", SUPPORTED_FORMATS)
def test_status_verbose_flag(flag: str, fmt: str) -> None:
    """Test that the --verbose flag adds extra fields to the output."""
    res = run_cli(["status", flag, "--format", fmt])
    data = load_output(res.stdout, fmt)
    assert "status" in data
    assert "python" in data
    assert "platform" in data


@pytest.mark.parametrize("fmt", SUPPORTED_FORMATS)
def test_status_default_output(fmt: str) -> None:
    """Test that the default output is pretty-printed."""
    res = run_cli(["status", "--format", fmt])
    assert res.returncode == 0
    if fmt == "json":
        assert res.stdout.count("\n") >= 1


@pytest.mark.parametrize("fmt", SUPPORTED_FORMATS)
def test_status_no_pretty_flag(fmt: str) -> None:
    """Test that the --no-pretty flag produces compact output."""
    res = run_cli(["status", "--format", fmt, "--no-pretty"])
    assert res.returncode == 0
    assert res.stdout.count("\n") <= 1


@pytest.mark.parametrize("flag", VALID_FLAGS["debug"])
def test_status_debug_flag(flag: str) -> None:
    """Test that the --debug flag produces verbose, pretty-printed output and diagnostics."""
    res = run_cli(["status", flag])
    assert res.returncode == 0
    assert res.stdout.count("\n") >= 2
    assert res.stderr.strip()
    data = json.loads(res.stdout)
    assert "status" in data


@pytest.mark.parametrize("flag", VALID_FLAGS["debug"])
def test_status_debug_forces_pretty(flag: str) -> None:
    """Test that --debug forces pretty-printing, ignoring --no-pretty."""
    res = run_cli(["status", flag, "--no-pretty"])
    assert res.returncode == 0
    assert res.stdout.count("\n") >= 2


@pytest.mark.parametrize("flag", VALID_FLAGS["help"])
def test_status_help_flag_strict(flag: str) -> None:
    """Test that the help flag shows well-formatted usage info."""
    res = run_cli(["status", flag])
    output = res.stdout
    assert res.returncode == 0
    assert output.lstrip().startswith("Usage:"), f"Help: {output[:40]!r}"
    assert output.count("\n") < 50, "Help output too long"
    for opt in ALL_FLAGS:
        assert opt in output, f"Help missing: {opt}"
    assert "status" in output
    assert not re.search(r"error|exception|traceback", output, re.I)


@pytest.mark.parametrize(
    "args",
    [
        ["--quiet", "--verbose"],
        ["-q", "-v"],
        ["--format", "yaml", "-q"],
        ["--verbose", "-f", "yaml"],
        ["--debug", "--no-pretty"],
    ],
)
def test_status_flag_combinations_success(args: list[str]) -> None:
    """Test that various flag combinations succeed."""
    res = run_cli(["status", *args])
    assert res.returncode == 0


@pytest.mark.parametrize(
    ("args", "expect_output"),
    [
        (["--quiet", "--verbose"], False),
        (["-q", "-v"], False),
        (["--format", "yaml", "-q"], False),
        (["--verbose", "-f", "yaml"], True),
        (["--debug", "--no-pretty"], True),
    ],
)
def test_status_flag_combinations_output_precedence(
    args: list[str], expect_output: bool
) -> None:
    """Test flag precedence for output presence (quiet wins)."""
    res = run_cli(["status", *args])
    assert bool(res.stdout.strip()) is expect_output


@pytest.mark.parametrize(
    ("flag", "value"),
    [("--format", "xml"), ("-f", ""), ("--format", "123")],
    ids=_flag_value_id,
)
def test_status_invalid_format_strict(flag: str, value: str) -> None:
    """Test that an invalid --format value fails correctly."""
    res = run_cli(["status", flag, value])
    assert res.returncode == 2
    target = res.stdout or res.stderr
    data = json.loads(target)
    assert "error" in data
    assert "format" in data["error"].lower()
    assert not re.search(r"traceback", data["error"], re.I)


def test_status_invalid_flag_contract() -> None:
    """Test that an invalid flag fails with a clear error."""
    res = run_cli(["status", "--notarealflag"])
    assert res.returncode != 0
    err = (res.stdout or res.stderr).lower()
    assert "no such option" in err or "unrecognized" in err or "error" in err


def test_status_non_ascii_argument() -> None:
    """Test that non-ASCII arguments are rejected."""
    res = run_cli(["status", "--foo", "café"])
    assert res.returncode != 0
    err = (res.stdout + res.stderr).lower()
    assert "ascii" in err or "encoding" in err or "error" in err


def test_status_non_ascii_env(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that non-ASCII environment variables are rejected."""
    monkeypatch.setenv("BIJUXCLI_STATUS", "näme")
    res = run_cli(["status"])
    assert res.returncode != 0
    err = (res.stdout + res.stderr).lower()
    assert "ascii" in err or "encoding" in err or "error" in err


def test_status_env_case_insensitivity(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the CLI ignores lowercase environment variables."""
    monkeypatch.setenv("bijuxcli_status", "bad")
    res = run_cli(["status"])
    data = json.loads(res.stdout)
    assert data["status"] == "ok"


def test_status_locale_c(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test correct operation with the 'C' locale."""
    monkeypatch.setenv("LANG", "C")
    res = run_cli(["status"])
    assert res.returncode == 0
    json.loads(res.stdout)


def test_status_idempotent_fields() -> None:
    """Test that deterministic fields are identical across runs."""
    runs = [json.loads(run_cli(["status", "-v"]).stdout) for _ in range(2)]
    for key in {"status", "python", "platform"}:
        assert runs[0].get(key) == runs[1].get(key)
    nondet = set(runs[0]) - {"status", "python", "platform", "version"}
    assert nondet <= {"timestamp", "uptime"}


@pytest.mark.parametrize("flag", [["--format", "json"], ["--verbose"], ["--debug"]])
def test_status_no_stacktrace_warning_leakage(flag: list[str]) -> None:
    """Test that no internal warnings or tracebacks leak into output."""
    res = run_cli(["status", *flag])
    out = (res.stdout + res.stderr).lower()
    assert "traceback" not in out
    assert "warning" not in out


@pytest.mark.parametrize(
    ("args", "expect"),
    [
        (["--format", "json", "--format", "yaml"], "yaml"),
        (["-f", "yaml", "-f", "json"], "json"),
    ],
)
def test_status_duplicate_flags(args: list[str], expect: str) -> None:
    """Test that the last of a duplicated flag is used."""
    res = run_cli(["status", *args])
    data = json.loads(res.stdout) if expect == "json" else yaml.safe_load(res.stdout)
    assert "status" in data


@pytest.mark.parametrize("flag", ["--format", "-f"])
def test_status_fuzzed_format(flag: str) -> None:
    """Test graceful failure with a fuzzed format value."""
    garbage = "".join(random.choices(string.printable, k=8))  # noqa: S311
    res = run_cli(["status", flag, garbage])
    assert res.returncode == 2
    data = json.loads(res.stdout or res.stderr)
    assert "error" in data


def test_command_performance_status() -> None:
    """Test that the status command executes quickly."""
    start = time.monotonic()
    res = run_cli(["status"])
    elapsed = time.monotonic() - start
    assert res.returncode == 0
    assert elapsed < 5.0, f"Too slow: {elapsed:.2f}s"


def test_status_parallel_invocations() -> None:
    """Test that parallel invocations do not interfere with each other."""
    results: list[Any] = []

    def run_and_store() -> None:
        results.append(run_cli(["status"]))

    threads = [threading.Thread(target=run_and_store) for _ in range(2)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    for res in results:
        assert res.returncode == 0
        assert "status" in json.loads(res.stdout)


def test_status_no_user_config_pollution(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that local user config files do not affect the status output."""
    config_path = tmp_path / "bijuxcli.toml"
    config_path.write_text("[main]\nfoo='bar'")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(config_path))
    res = run_cli(["status"])
    data = json.loads(res.stdout)
    assert "status" in data
    assert "foo" not in data


def test_status_watch_json_ok() -> None:
    """Test --watch JSON emits ticks and a stop message."""
    _, lines, remainder = run_cli_watch(["--watch", "1.0", "--format", "json"])
    raw = "".join(lines) + remainder
    outputs, errors = [], []
    for m in re.finditer(r"\{.*?\}", raw, flags=re.S):
        chunk = m.group()
        try:
            outputs.append(json.loads(chunk))
        except Exception as e:
            errors.append((chunk, repr(e)))
    if not outputs:
        raise AssertionError(f"No outputs parsed. Raw output:\n{raw}\nErrors: {errors}")
    if errors:
        raise AssertionError(f"JSON decode errors: {errors}")
    assert any(
        o.get("status") == "ok" and isinstance(o.get("ts"), float) for o in outputs
    ), f"No tick entries found in {outputs!r}"
    assert any(o.get("status") == "watch-stopped" for o in outputs), (
        f"No stop entry found in {outputs!r}"
    )


@st.composite
def status_flag_permutations(draw: Any) -> list[str]:
    """Generate permutations of status command flags for hypothesis testing."""
    flags = cast(
        list[str],
        draw(
            st.lists(st.sampled_from(ALL_FLAGS), min_size=0, max_size=6, unique=False)
        ),
    )
    if any(f in flags for f in ("--format", "-f")):
        try:
            idx = flags.index("--format")
        except ValueError:
            idx = flags.index("-f")
        flags = (
            flags[: idx + 1]
            + [cast(str, draw(st.sampled_from(FORMATS)))]
            + flags[idx + 1 :]
        )
    return flags


def test_command_performance_status_under_load() -> None:
    """Test that repeated status calls are consistently fast."""
    times = []
    for _ in range(10):
        t0 = time.perf_counter()
        res = run_cli(["status"])
        t1 = time.perf_counter()
        assert res.returncode == 0
        assert "status" in json.loads(res.stdout)
        times.append(t1 - t0)
    max_time = max(times)
    assert max_time < 5.0, f"Max single status call too slow: {max_time:.2f}s"


def test_status_watch_yaml_rejected() -> None:
    """Test that --watch rejects non-JSON formats like YAML."""
    proc = subprocess.run(  # noqa: S603
        [
            sys.executable,
            "-m",
            "bijux_cli",
            "status",
            "--watch",
            "0.05",
            "--format",
            "yaml",
        ],
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 2
    err = proc.stdout + proc.stderr
    assert "only json output is supported" in err.lower()
    assert "error" in err.lower()


def test_status_watch_invalid_interval() -> None:
    """Test that --watch rejects invalid time intervals."""
    proc = subprocess.run(  # noqa: S603
        [sys.executable, "-m", "bijux_cli", "status", "--watch", "0"],
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 2
    err = proc.stdout + proc.stderr
    assert "invalid watch interval" in err.lower()
    assert "error" in err.lower()


def assert_error_contract(stdout: str, stderr: str) -> None:
    """Assert that an error output follows the expected JSON contract."""
    try:
        data = json.loads(stdout or stderr)
    except Exception:
        data = None
    assert data is not None
    assert "error" in data
    assert "code" in data


@given(flags=status_flag_permutations())
@settings(max_examples=10, deadline=None, suppress_health_check=[HealthCheck.too_slow])
def test_status_hypothesis_flags(flags: list[str]) -> None:
    """Test various flag combinations generated by hypothesis."""
    for i, flag in enumerate(flags):
        if flag in ("--format", "-f") and (
            i + 1 == len(flags) or flags[i + 1].startswith("-")
        ):
            return

    res = run_cli(["status", *flags])

    if "-h" in flags or "--help" in flags:
        assert res.returncode == 0
        assert res.stdout.lstrip().startswith("Usage:")
        return

    if "-q" in flags or "--quiet" in flags:
        assert not res.stdout.strip()
        assert not res.stderr.strip()
        return

    for i, flag in enumerate(flags):
        if flag in ("--format", "-f"):
            val = flags[i + 1].lower()
            if val not in ("json", "yaml"):
                assert res.returncode != 0
                return

    assert res.returncode == 0
    text = res.stdout.strip()
    assert text, "Expected output but got none"
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        payload = yaml.safe_load(text)
    assert isinstance(payload, dict)
    assert "status" in payload


@settings(max_examples=10, deadline=None)
@given(
    nonascii=st.text(
        alphabet=st.characters(
            blacklist_categories=["C", "Z", "M", "N"],
            blacklist_characters=[chr(i) for i in range(128)],
        ),
        min_size=1,
        max_size=10,
    )
)
def test_status_non_ascii_arg_fuzz(nonascii: str) -> None:
    """Test rejection of fuzzed non-ASCII arguments."""
    res = run_cli(["status", "--foo", nonascii])
    assert res.returncode != 0
    assert_error_contract(res.stdout, res.stderr)


def test_status_watch_golden() -> None:
    """Golden test for watch output (≥1 tick + stop)."""
    _, lines, remainder = run_cli_watch(["--watch", "0.1", "--format", "json"])
    raw = "".join(lines) + remainder
    decoder = json.JSONDecoder()
    objs: list[dict[str, Any]] = []
    s = raw
    idx = 0
    length = len(s)
    while idx < length:
        while idx < length and s[idx].isspace():
            idx += 1
        if idx >= length:
            break

        obj, end = decoder.raw_decode(s, idx)
        objs.append(obj)
        idx = end
    ticks = [
        o for o in objs if o.get("status") == "ok" and isinstance(o.get("ts"), float)
    ]
    stops = [o for o in objs if o.get("status") == "watch-stopped"]
    assert len(ticks) >= 1, f"Expected at least one tick, got: {objs!r}"
    assert len(stops) >= 1, f"Expected a stop message, got: {objs!r}"


@given(
    interval=st.floats(
        min_value=0, max_value=1.5, allow_nan=False, allow_infinity=False
    )
)
@settings(max_examples=10, deadline=None)
def test_status_watch_fuzz_interval(interval: float) -> None:
    """Fuzz watch interval for validation."""
    if interval <= 0:
        res = run_cli(["status", "--watch", str(interval)])
        assert res.returncode == 2
    else:
        proc, _, _ = run_cli_watch(["--watch", str(interval)])
        assert proc.returncode == 0


def test_status_watch_memory_leak() -> None:
    """Watch mode should not leak memory over time."""
    proc = psutil.Process()
    mem_before = proc.memory_info().rss
    _, _, _ = run_cli_watch(["--watch", "0.1"], timeout=1.0)
    mem_after = proc.memory_info().rss
    assert mem_after - mem_before < 5 * 1024 * 1024


def test_status_help_precedes_and_ignores_other_flag_errors() -> None:
    """ADR Test: --help must short-circuit and ignore errors from other invalid flags."""
    res = run_cli(["status", "--help", "--format"])

    assert res.returncode == 0, (
        "ADR Violation: --help did not short-circuit an invalid flag error."
    )
    assert "usage: bijux status" in res.stdout.lower()
    assert not res.stderr.strip()

    res_2 = run_cli(["status", "-h", "--unsupported-flag"])
    assert res_2.returncode == 0, (
        "ADR Violation: --help did not short-circuit an unknown flag error."
    )
    assert "usage: bijux status" in res_2.stdout.lower()


def test_status_help_precedes_di_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    """ADR Test: --help must short-circuit everything, even a DI container failure."""
    from typing import NoReturn

    def mock_resolve_fails(self: Any, protocol: Any) -> NoReturn:
        """A mock DI resolve method that raises a RuntimeError to test failure handling."""
        raise RuntimeError("Simulated DI Container Crash")

    from bijux_cli.core.di import DIContainer

    monkeypatch.setattr(DIContainer, "resolve", mock_resolve_fails)

    res = run_cli(["status", "--help"])

    assert res.returncode == 0, (
        "ADR Violation: --help did not short-circuit a DI failure."
    )
    assert "usage: bijux status" in res.stdout.lower()
    assert not res.stderr.strip()
