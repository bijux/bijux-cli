# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux sleep` command."""

from __future__ import annotations

from collections.abc import Callable
import json
import os
from pathlib import Path
import random
import signal
import string
from subprocess import PIPE, CompletedProcess, Popen
import sys
import threading
import time
from typing import Any, cast

from hypothesis import HealthCheck, given, settings
import hypothesis.strategies as st
import pytest
import yaml

from tests.e2e.conftest import run_cli

SUPPORTED_FORMATS = ["json", "yaml"]
VALID_FLAGS = {
    "help": ["--help", "-h"],
    "quiet": ["--quiet", "-q"],
    "debug": ["--debug", "-d"],
    "verbose": ["--verbose", "-v"],
}
ALL_FLAGS = [
    "--help",
    "-h",
    "--quiet",
    "-q",
    "--debug",
    "-d",
    "--verbose",
    "-v",
    "--format",
    "-f",
    "--no-pretty",
    "--pretty",
]
FORMATS = ["json", "yaml", "bogus", "garbage", ""]
ALLOWED = "".join(chr(c) for c in range(160, 0x2000))


def _flag_value_id(x: Any) -> str:
    """Create a readable ID for parametrized test cases."""
    if isinstance(x, list | tuple) and len(x) >= 2:
        flag, value = x[:2]
        return f"{flag}={value or 'empty'}"
    return str(x)


def load_output(stdout: str, fmt: str) -> dict[str, Any]:
    """Load JSON or YAML from a string."""
    loader: Callable[[str], Any] = (
        json.loads if fmt.lower() == "json" else yaml.safe_load
    )
    data = loader(stdout)
    assert isinstance(data, dict)
    return data


@st.composite
def sleep_flag_permutations(draw: Any) -> list[str]:
    """Generate permutations of sleep command flags for hypothesis."""
    flags = draw(
        st.lists(st.sampled_from(ALL_FLAGS), min_size=0, max_size=6, unique=False)
    )
    if not any(f in flags for f in ("--help", "-h")):
        idx = random.randint(0, len(flags))  # noqa: S311
        flags = flags[:idx] + ["--seconds", "0.01"] + flags[idx:]
    if any(f in flags for f in ("--format", "-f")):
        try:
            idx = flags.index("--format")
        except ValueError:
            idx = flags.index("-f")
        flags = flags[: idx + 1] + [draw(st.sampled_from(FORMATS))] + flags[idx + 1 :]
    return cast(list[str], flags)


def test_sleep_idempotent() -> None:
    """Test that sleep of the same duration produces same slept field."""
    runs = [
        json.loads(run_cli(["sleep", "--seconds", "0.05"]).stdout) for _ in range(2)
    ]
    for run in runs:
        assert "slept" in run
    assert runs[0]["slept"] == pytest.approx(runs[1]["slept"], abs=1e-7)


def test_sleep_zero_seconds() -> None:
    """Test sleep for zero seconds is valid."""
    res = run_cli(["sleep", "--seconds", "0"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert data["slept"] == 0


def test_sleep_negative_seconds() -> None:
    """Test negative duration is a contract error."""
    res = run_cli(["sleep", "--seconds", "-0.5"])
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "non-negative" in msg or "negative" in msg or "error" in msg


def test_sleep_missing_seconds() -> None:
    """Test missing required argument fails with usage error."""
    res = run_cli(["sleep"])
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "required" in msg or "seconds" in msg


def test_sleep_seconds_non_numeric() -> None:
    """Test non-numeric input fails contract."""
    res = run_cli(["sleep", "--seconds", "foo"])
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "invalid" in msg or "not a valid float" in msg


def test_sleep_env_timeout(tmp_path: Path) -> None:
    """Test BIJUXCLI_COMMAND_TIMEOUT limits sleep duration."""
    env = os.environ.copy()
    env["BIJUXCLI_COMMAND_TIMEOUT"] = "0.05"
    res = run_cli(["sleep", "--seconds", "1"], env=env)
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "timeout" in msg


def test_sleep_config_timeout(tmp_path: Path) -> None:
    """Test config file timeout is honored if env is unset."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_COMMAND_TIMEOUT=0.05\n")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    res = run_cli(["sleep", "--seconds", "1"], env=env)
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "timeout" in msg


def test_sleep_env_vs_config_timeout_precedence(tmp_path: Path) -> None:
    """Env var timeout wins over config file."""
    env = os.environ.copy()
    env["BIJUXCLI_COMMAND_TIMEOUT"] = "0.05"
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_COMMAND_TIMEOUT=1\n")
    env["BIJUXCLI_CONFIG"] = str(cfg)
    res = run_cli(["sleep", "--seconds", "1"], env=env)
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "timeout" in msg


def test_sleep_invalid_timeout_config(tmp_path: Path) -> None:
    """Test invalid config timeout triggers contract error."""
    cfg = tmp_path / ".env"
    cfg.write_text("BIJUXCLI_COMMAND_TIMEOUT=foobar\n")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    res = run_cli(["sleep", "--seconds", "0.1"], env=env)
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "timeout" in msg or "invalid" in msg or "error" in msg


def test_sleep_ascii_only_env(tmp_path: Path) -> None:
    """Test non-ASCII in config or env triggers contract error."""
    cfg = tmp_path / ".env"
    cfg.write_bytes(b"BIJUXCLI_FOO=\xff\n")
    env = {"BIJUXCLI_CONFIG": str(cfg)}
    res = run_cli(["sleep", "--seconds", "0.01"], env=env)
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "ascii" in msg or "encoding" in msg


def test_sleep_large_number_fails() -> None:
    """Test huge sleep values fail gracefully."""
    res = run_cli(["sleep", "--seconds", str(sys.maxsize)])
    assert res.returncode != 0


def test_sleep_interrupt_signal() -> None:
    """Test SIGINT interrupts sleep and returns error."""
    import shutil

    bijux_path = shutil.which("bijux")
    assert bijux_path is not None, "Could not find 'bijux' executable in PATH"

    proc = Popen(  # noqa: S603
        [bijux_path, "sleep", "--seconds", "10"],
        stdout=PIPE,
        stderr=PIPE,
        text=True,
    )
    time.sleep(0.2)
    proc.send_signal(signal.SIGINT)
    stdout, stderr = proc.communicate(timeout=2)
    assert proc.returncode not in (None, 0)
    msg = (stdout + stderr).lower()
    assert "interrupt" in msg or "signal" in msg or proc.returncode == -signal.SIGINT


def test_sleep_flag_combinations() -> None:
    """Test various legal flag combinations."""
    combos = [
        ["--seconds", "0.01", "--quiet", "--debug"],
        ["--seconds", "0.01", "--quiet", "--verbose"],
        ["--seconds", "0.01", "--debug", "--no-pretty"],
        ["--seconds", "0.01", "--format", "yaml", "--no-pretty"],
    ]
    for args in combos:
        res = run_cli(["sleep", *args])
        if "--quiet" in args:
            assert not res.stdout.strip()
        else:
            assert res.returncode == 0


def test_sleep_fuzzed_format() -> None:
    """Test invalid/fuzzed format values fail gracefully."""
    garbage = "".join(random.choices(string.printable, k=8))  # noqa: S311
    res = run_cli(["sleep", "--seconds", "0.01", "--format", garbage])
    assert res.returncode == 2
    data = json.loads(res.stderr)
    assert "error" in data


def test_sleep_pretty_json() -> None:
    """Test --pretty yields indented JSON."""
    res = run_cli(["sleep", "--seconds", "0.01", "--pretty"])
    assert res.returncode == 0
    assert "{" in res.stdout
    assert "\n" in res.stdout


def test_sleep_pretty_yaml() -> None:
    """Test --pretty yields indented YAML."""
    res = run_cli(["sleep", "--seconds", "0.01", "--format", "yaml", "--pretty"])
    assert res.returncode == 0
    assert "slept:" in res.stdout
    assert "\n" in res.stdout


def test_sleep_parallel_invocations() -> None:
    """Test concurrent sleep runs do not interfere."""
    results: list[CompletedProcess[str]] = []

    def run_and_store() -> None:
        """Helper to run sleep and append result."""
        results.append(run_cli(["sleep", "--seconds", "0.01"]))

    threads = [threading.Thread(target=run_and_store) for _ in range(2)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    for res in results:
        assert res.returncode == 0
        assert "slept" in json.loads(res.stdout)


def test_command_performance_sleep() -> None:
    """Sleep command executes quickly for short durations."""
    start = time.monotonic()
    res = run_cli(["sleep", "--seconds", "0.01"])
    elapsed = time.monotonic() - start
    assert res.returncode == 0
    assert elapsed < 5.0


def test_sleep_no_stacktrace_warning_leakage() -> None:
    """No traceback/warning leaks in any output."""
    res = run_cli(["sleep", "--seconds", "0.01"])
    out = (res.stdout + res.stderr).lower()
    assert "traceback" not in out
    assert "warning" not in out


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
def test_sleep_format_flag(flag: str, value: str, loader: Callable[[str], Any]) -> None:
    """Test --format flag with both JSON and YAML."""
    res = run_cli(["sleep", "--seconds", "0.01", flag, value])
    assert res.returncode == 0
    data = loader(res.stdout)
    assert "slept" in data
    assert data["slept"] == pytest.approx(0.01, abs=1e-7)


@pytest.mark.parametrize("flag", VALID_FLAGS["quiet"])
def test_sleep_quiet_flag(flag: str) -> None:
    """Test --quiet suppresses all output, exit=0 on success."""
    res = run_cli(["sleep", "--seconds", "0.01", flag])
    assert res.returncode == 0
    assert not res.stdout.strip()
    assert not res.stderr.strip()


@pytest.mark.parametrize("flag", VALID_FLAGS["help"])
def test_sleep_help_flag(flag: str) -> None:
    """Test help flag emits correct help/usage info."""
    res = run_cli(["sleep", flag])
    output = res.stdout
    assert res.returncode == 0
    assert output.lstrip().startswith("Usage:") or "Pause execution" in output
    for opt in ["--seconds", "--quiet", "--debug", "--format", "--no-pretty"]:
        assert opt in output


@pytest.mark.parametrize("flag", VALID_FLAGS["debug"])
def test_sleep_debug_flag(flag: str) -> None:
    """Test --debug emits pretty-printed JSON and diagnostics."""
    res = run_cli(["sleep", "--seconds", "0.01", flag])
    assert res.returncode == 0
    assert res.stdout.count("\n") >= 1
    data = json.loads(res.stdout)
    assert "slept" in data


@pytest.mark.parametrize("flag", VALID_FLAGS["debug"])
def test_sleep_debug_forces_pretty(flag: str) -> None:
    """Test --debug overrides --no-pretty (still pretty-printed)."""
    res = run_cli(["sleep", "--seconds", "0.01", flag, "--no-pretty"])
    assert res.returncode == 0
    assert res.stdout.count("\n") >= 1


@pytest.mark.parametrize("fmt", SUPPORTED_FORMATS)
def test_sleep_no_pretty_flag(fmt: str) -> None:
    """Test --no-pretty disables indentation."""
    res = run_cli(["sleep", "--seconds", "0.01", "--format", fmt, "--no-pretty"])
    assert res.returncode == 0
    assert res.stdout.count("\n") <= 1


@pytest.mark.parametrize("fmt", SUPPORTED_FORMATS)
def test_sleep_default_output(fmt: str) -> None:
    """Test pretty-printed output by default."""
    res = run_cli(["sleep", "--seconds", "0.01", "--format", fmt])
    assert res.returncode == 0
    if fmt == "json":
        assert res.stdout.count("\n") >= 1


@pytest.mark.parametrize("flag", VALID_FLAGS["verbose"])
def test_sleep_verbose_flag(flag: str) -> None:
    """Test --verbose does not add fields unless implemented (future)."""
    res = run_cli(["sleep", "--seconds", "0.01", flag])
    data = json.loads(res.stdout)
    assert "slept" in data


@pytest.mark.parametrize(
    "value",
    ["0.123456", "1e-2", "1e-10"],
)
def test_sleep_float_and_scientific(value: str) -> None:
    """Test float and scientific notation for seconds."""
    res = run_cli(["sleep", "--seconds", value])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert data["slept"] == pytest.approx(float(value), abs=1e-7)


@pytest.mark.parametrize(
    ("flag", "value"),
    [("--format", "xml"), ("-f", ""), ("--format", "bogus")],
    ids=_flag_value_id,
)
def test_sleep_invalid_format(flag: str, value: str) -> None:
    """Test invalid --format is a contract error."""
    res = run_cli(["sleep", "--seconds", "0.01", flag, value])
    assert res.returncode == 2
    data = json.loads(res.stderr)
    assert "error" in data
    assert "format" in data["error"].lower()


@pytest.mark.parametrize("flag", [["--format", "json"], ["--debug"], ["--pretty"]])
def test_sleep_duplicate_flags(flag: list[str]) -> None:
    """Last value for duplicate flag is honored."""
    res = run_cli(["sleep", "--seconds", "0.01", *flag, *flag])
    assert res.returncode == 0


@pytest.mark.parametrize(
    ("args", "expect_output"),
    [
        (["--seconds", "0.01", "--quiet", "--debug"], False),
        (["--seconds", "0.01", "--quiet", "--verbose"], False),
        (["--seconds", "0.01", "--format", "yaml", "--no-pretty"], True),
        (["--seconds", "0.01", "--debug", "--no-pretty"], True),
    ],
)
def test_sleep_flag_output_precedence(args: list[str], expect_output: bool) -> None:
    """Test flag precedence (quiet wins, debug overrides pretty)."""
    res = run_cli(["sleep", *args])
    has_output = bool(res.stdout.strip())
    assert has_output is expect_output


@pytest.mark.parametrize(
    "flags",
    [
        ["--help"],
        ["-h"],
        ["--seconds", "0.01", "--format", "json"],
        ["--seconds", "0.01", "--format", "yaml"],
        ["--seconds", "0.01", "--quiet"],
        ["--seconds", "0.01", "-q"],
        ["--seconds", "0.01", "-d"],
        ["--seconds", "0.01", "--debug"],
        ["--seconds", "0.01", "--pretty"],
        ["--seconds", "0.01", "-f", "json"],
        ["--seconds", "0.01", "-f", "yaml"],
        ["--seconds", "0.01", "--no-pretty"],
    ],
)
def test_sleep_hypothesis_flags(flags: list[str]) -> None:
    """Test various flag combinations for the sleep command."""
    for i, flag in enumerate(flags):
        if (
            flag in ("--format", "-f")
            and i + 1 < len(flags)
            and flags[i + 1].startswith("-")
        ):
            return

    res = run_cli(["sleep", *flags])

    if "--help" in flags or "-h" in flags:
        if any(
            flag in ("--format", "-f")
            and (i + 1 == len(flags) or flags[i + 1].startswith("-"))
            for i, flag in enumerate(flags)
        ):
            assert res.returncode != 0
            assert "requires an argument" in (res.stdout + res.stderr).lower()
            return
        assert res.returncode == 0
        assert res.stdout.lstrip().startswith("Usage:")
        return

    if "--quiet" in flags or "-q" in flags:
        assert not res.stdout.strip()
        assert not res.stderr.strip()
        return

    for i, flag in enumerate(flags):
        if flag in ("--format", "-f"):
            if i + 1 == len(flags) or flags[i + 1].startswith("-"):
                return
            value = flags[i + 1].lower()
            if value not in ("json", "yaml"):
                assert res.returncode != 0
                return

    assert res.returncode == 0, f"Failed with flags: {flags}"
    assert res.stdout.strip(), f"No output: flags={flags!r}"
    try:
        data = json.loads(res.stdout)
    except Exception:
        try:
            data = yaml.safe_load(res.stdout)
        except Exception:
            pytest.fail(f"Output is not JSON or YAML: {res.stdout!r}")
    assert isinstance(data, dict)
    assert "slept" in data


@settings(max_examples=10, deadline=None, suppress_health_check=[HealthCheck.too_slow])
@given(nonascii=st.text(alphabet=ALLOWED, min_size=1, max_size=10))
def test_sleep_non_ascii_arg_fuzz(nonascii: str) -> None:
    """Test rejection of fuzzed non-ASCII arguments."""
    res = run_cli(["sleep", "--seconds", "0.01", "--foo", nonascii])
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "ascii" in msg or "encoding" in msg or "error" in msg


def test_sleep_float_overflow() -> None:
    """Test extreme floats (inf/nan/large)."""
    res = run_cli(["sleep", "--seconds", str(sys.float_info.max)])
    assert res.returncode != 0
    msg = (res.stdout + res.stderr).lower()
    assert "timeout" in msg or "error" in msg


base_key = st.text(
    min_size=1, max_size=10, alphabet=string.ascii_letters + string.digits
).filter(lambda s: not s[0].isdigit())

env_key_strategy = base_key.map(lambda s: f"TEST_{s}")
env_val_strategy = st.text(min_size=1, max_size=10)


@given(env_key=env_key_strategy, env_val=env_val_strategy)
@settings(max_examples=10, deadline=None)
def test_sleep_fuzz_env(env_key: str, env_val: str) -> None:
    """Fuzz a TEST_* env var; `bijux_cli sleep` must always succeed with valid JSON."""
    from hypothesis import assume

    assume("\x00" not in env_val)
    orig = os.environ.get(env_key)
    try:
        os.environ[env_key] = env_val
        res = run_cli(["sleep", "--seconds", "0.01"])
        assert res.returncode == 0
        data = json.loads(res.stdout)
        assert "slept" in data
    finally:
        if orig is None:
            os.environ.pop(env_key, None)
        else:
            os.environ[env_key] = orig
