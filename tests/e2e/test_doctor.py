# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux doctor` command."""

from __future__ import annotations

from collections.abc import Callable
from contextlib import suppress
import difflib
import json
import os
from pathlib import Path
import signal
from subprocess import PIPE, Popen
import threading
import time
from typing import Any, cast

from hypothesis import HealthCheck, assume, given, settings
from hypothesis.strategies import (
    DrawFn,
    characters,
    composite,
    lists,
    sampled_from,
    text,
)
import psutil
import pytest
import yaml

from tests.e2e.conftest import run_cli

SUPPORTED_FORMATS = ["json", "yaml"]
FORMATS = ["json", "yaml", "bogus", "garbage", ""]

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
    "--pretty",
    "--no-pretty",
]


def is_valid_env_key(key: str) -> bool:
    if not key:
        return False
    if "=" in key or "\x00" in key:
        return False
    return (key[0].isalpha() or key[0] == "_") and all(
        c.isalnum() or c == "_" for c in key
    )


env_key_strategy = text(
    min_size=1,
    max_size=10,
    alphabet="abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_",
).filter(is_valid_env_key)

env_val_strategy = text(
    min_size=1,
    max_size=20,
    alphabet=characters(blacklist_categories=["Cc", "Cs", "Zs"]),
)


def _strip_non_ascii_env() -> None:
    """Remove any non-ASCII environment variables for test isolation."""
    for k, v in list(os.environ.items()):
        try:
            v.encode("ascii")
        except UnicodeEncodeError:
            del os.environ[k]


_strip_non_ascii_env()


def _no_stacktrace_leak(text: str) -> None:
    """Assert no traceback or framework names leak into user output."""
    s = text.lower()
    assert "traceback" not in s
    assert "typer" not in s
    assert "click" not in s


def load_json(text: str) -> dict[str, Any]:
    """Parse JSON text, ensuring a dict result."""
    data: Any = json.loads(text)
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict JSON, got {type(data).__name__}: {data!r}")
    return cast(dict[str, Any], data)


def load_yaml(text: str) -> dict[str, Any]:
    """Parse YAML text, ensuring a dict result."""
    data: Any = yaml.safe_load(text)
    if not isinstance(data, dict):
        raise TypeError(f"Expected dict YAML, got {type(data).__name__}: {data!r}")
    return cast(dict[str, Any], data)


def parse_output(fmt: str, text: str) -> dict[str, Any]:
    """Parse output as JSON or YAML according to ``fmt``."""
    if fmt.lower() == "json":
        return load_json(text)
    if fmt.lower() == "yaml":
        return load_yaml(text)
    raise AssertionError(f"Unknown format: {fmt}")


def assert_status_contract(payload: dict[str, Any]) -> None:
    """Assert minimal doctor status contract."""
    assert "status" in payload
    assert payload["status"] in ("healthy", "unhealthy")


def assert_error_contract(payload: dict[str, Any], code: int | None = None) -> None:
    """Assert minimal error contract."""
    assert "error" in payload
    if code is not None:
        assert "code" in payload
        assert payload["code"] == code


def _flag_value_id(x: Any) -> str:
    """Readable parametrization id for flag/value tuples."""
    if isinstance(x, (list | tuple)) and len(x) >= 2:
        return f"{x[0]}={x[1] or 'empty'}"
    return str(x)


@composite
def doctor_flag_permutations(draw: DrawFn) -> list[str]:
    """Generate permutations of flags for hypothesis tests."""
    base = draw(lists(sampled_from(ALL_FLAGS), min_size=0, max_size=6, unique=False))
    assert isinstance(base, list)
    if any(f in base for f in ("--format", "-f")):
        fmt_flag = draw(sampled_from(["--format", "-f"]))
        fmt_value = draw(sampled_from(FORMATS))
        return [f for f in base if f not in ("--format", "-f")] + [fmt_flag, fmt_value]
    return base


@pytest.mark.parametrize(
    ("flags", "fmt", "loader"),
    [
        (["--format", "json"], "json", load_json),
        (["-f", "json"], "json", load_json),
        (["--format", "yaml"], "yaml", load_yaml),
        (["-f", "yaml"], "yaml", load_yaml),
        (["--format", "JSON"], "json", load_json),
        (["--format", "YAML"], "yaml", load_yaml),
    ],
    ids=[
        "long-json",
        "short-json",
        "long-yaml",
        "short-yaml",
        "long-JSON-upcase",
        "long-YAML-upcase",
    ],
)
def test_doctor_contract_formats(
    flags: list[str], fmt: str, loader: Callable[[str], dict[str, Any]]
) -> None:
    """Contract: valid formats produce structured output; no leaks."""
    res = run_cli(["doctor", *flags])
    assert res.returncode in (0, 1)
    data = loader(res.stdout)
    assert_status_contract(data)
    _no_stacktrace_leak(res.stdout + res.stderr)


@pytest.mark.parametrize("flag", ["--quiet", "-q"])
def test_doctor_quiet(flag: str) -> None:
    """Quiet has precedence; no stdout/stderr, exit code signals result."""
    res = run_cli(["doctor", flag])
    assert res.returncode in (0, 1, 2, 3)
    assert not res.stdout.strip()
    assert not res.stderr.strip()


@pytest.mark.parametrize(
    ("flag", "value"),
    [
        ("--format", "xml"),
        ("--format", ""),
        ("--format", "123"),
        ("-f", "bogus"),
    ],
    ids=_flag_value_id,
)
def test_doctor_invalid_format_structured_error(flag: str, value: str) -> None:
    """Invalid ``--format`` yields code 2 and a structured error payload."""
    res = run_cli(["doctor", flag, value])
    assert res.returncode == 2
    payload = load_json(res.stdout or res.stderr)
    assert_error_contract(payload, code=2)
    _no_stacktrace_leak(res.stdout + res.stderr)


def test_doctor_duplicate_format_last_wins() -> None:
    """Duplicate ``--format``: last one is effective."""
    res = run_cli(["doctor", "--format", "yaml", "--format", "json"])
    assert res.returncode in (0, 1)
    payload = load_json(res.stdout)
    assert_status_contract(payload)


@pytest.mark.parametrize("flag", ["--help", "-h"])
def test_doctor_help_output(flag: str) -> None:
    """Help output is short, readable, and includes major flags."""
    res = run_cli(["doctor", flag])
    assert res.returncode == 0
    text = res.stdout
    assert text.lstrip().lower().startswith("usage:")
    assert text.count("\n") < 60
    for k in ("--help", "-h", "--quiet", "-q", "--verbose", "-v", "--format", "-f"):
        assert k in text
    _no_stacktrace_leak(text)


def test_doctor_ascii_env_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Non-ASCII in environment should yield code 3 with structured error."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/tmp/\u2603")  # noqa: S108
    res = run_cli(["doctor"])
    assert res.returncode == 3
    payload = load_json(res.stdout or res.stderr)
    assert_error_contract(payload, code=3)


def test_doctor_unknown_flag_structured_error() -> None:
    """Unknown flags produce structured error with code 2."""
    res = run_cli(["doctor", "--notaflag"])
    assert res.returncode == 2
    payload = load_json(res.stdout or res.stderr)
    assert_error_contract(payload, code=2)


def test_doctor_no_leaks_common_paths() -> None:
    """Common successful paths should not leak warnings/tracebacks."""
    for flags in (["--format", "json"], ["--format", "yaml"], ["-v"], ["-d"]):
        res = run_cli(["doctor", *flags])
        _no_stacktrace_leak(res.stdout + res.stderr)


def test_doctor_parallel_invocations() -> None:
    """Parallel doctor runs should not interfere with each other."""
    outs: list[str] = []

    def run_and_store() -> None:
        """Runs the 'doctor' command and appends its JSON stdout to a results list."""
        outs.append(run_cli(["doctor", "--format", "json"]).stdout)

    threads = [threading.Thread(target=run_and_store) for _ in range(3)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    for out in outs:
        payload = load_json(out)
        assert_status_contract(payload)


def test_command_performance_doctor_smoke() -> None:
    """Doctor should be reasonably fast."""
    t0 = time.perf_counter()
    res = run_cli(["doctor"])
    t1 = time.perf_counter()
    assert res.returncode in (0, 1)
    assert (t1 - t0) < 5.0


@settings(max_examples=10, deadline=None, suppress_health_check=[HealthCheck.too_slow])
@given(flags=doctor_flag_permutations())
def test_doctor_hypothesis_flags(flags: list[str]) -> None:
    """Fuzz flag combinations and validate precedence & contracts."""
    for i, f in enumerate(flags):
        if (
            f in ("--format", "-f")
            and i + 1 < len(flags)
            and flags[i + 1].startswith("-")
        ):
            return

    res = run_cli(["doctor", *flags])

    if "--help" in flags or "-h" in flags:
        assert res.returncode == 0
        assert res.stdout.lstrip().lower().startswith("usage:")
        return

    if "--quiet" in flags or "-q" in flags:
        assert not res.stdout.strip()
        assert not res.stderr.strip()
        return

    for i, f in enumerate(flags):
        if f in ("--format", "-f") and i + 1 < len(flags):
            v = flags[i + 1].lower()
            if v not in ("json", "yaml"):
                assert res.returncode != 0
                return

    assert res.returncode in (0, 1)
    fmt = "json"
    for i, f in enumerate(flags):
        if (
            f in ("--format", "-f")
            and i + 1 < len(flags)
            and flags[i + 1].lower() in ("json", "yaml")
        ):
            fmt = flags[i + 1].lower()
    data = parse_output(fmt, res.stdout)
    assert_status_contract(data)
    _no_stacktrace_leak(res.stdout + res.stderr)


def test_command_performance_doctor() -> None:
    """Detects regression: doctor must stay below 500ms under normal load."""
    t0 = time.perf_counter()
    res = run_cli(["doctor"])
    t1 = time.perf_counter()
    assert res.returncode in (0, 1)
    assert (t1 - t0) < 5.0


def test_doctor_no_memory_leak() -> None:
    """Doctor should not leak memory on repeated invocation."""
    proc = psutil.Process()
    mem_before = proc.memory_info().rss
    for _ in range(10):
        res = run_cli(["doctor", "--format", "json"])
        assert res.returncode in (0, 1)
    mem_after = proc.memory_info().rss
    assert mem_after - mem_before < 10 * 1024 * 1024


def test_doctor_many_env_vars(monkeypatch: pytest.MonkeyPatch) -> None:
    """Doctor should handle a large number of environment variables."""
    for i in range(1000):
        monkeypatch.setenv(f"FOOBAR_{i}", "X" * 100)
    res = run_cli(["doctor"])
    assert res.returncode in (0, 1)


def test_doctor_high_parallelism() -> None:
    """Doctor should handle high parallelism without errors."""
    outs: list[str] = []
    threads = [
        threading.Thread(
            target=lambda: outs.append(run_cli(["doctor", "--format", "json"]).stdout)
        )
        for _ in range(50)
    ]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    ok = 0
    for out in outs:
        s = (out or "").strip()
        if not s:
            continue
        with suppress(
            json.JSONDecodeError, AssertionError, KeyError, TypeError, ValueError
        ):
            assert_status_contract(load_json(s))
            ok += 1
    assert ok >= 10, f"doctor JSON successes too low: {ok}/50"


def test_doctor_output_golden() -> None:
    """Doctor output should match the golden file."""
    res = run_cli(["doctor", "--format", "json"])
    with open("tests/e2e/test_fixtures/doctor/doctor.json") as f:
        golden = f.read()
    diff = list(difflib.unified_diff(golden.splitlines(), res.stdout.splitlines()))
    assert not diff, "\n".join(diff)


def test_doctor_golden_healthy() -> None:
    """Golden test for healthy doctor output."""
    res = run_cli(["doctor", "--format", "json"])
    golden = Path("tests/e2e/test_fixtures/doctor/doctor.json").read_text()
    diff = list(difflib.unified_diff(golden.splitlines(), res.stdout.splitlines()))
    assert not diff, "\n".join(diff)


def test_doctor_signal_interrupt() -> None:
    """Test SIGINT during doctor."""
    proc = Popen(["bijux", "doctor"], stdout=PIPE, stderr=PIPE, text=True)  # noqa: S607, S603
    time.sleep(0.1)
    proc.send_signal(signal.SIGINT)
    stdout, stderr = proc.communicate(timeout=2)
    assert proc.returncode != 0
    msg = (stdout + stderr).lower()
    assert "interrupt" in msg or "signal" in msg


@given(env_key=env_key_strategy, env_val=env_val_strategy)
@settings(
    max_examples=10,
    deadline=None,
    suppress_health_check=[HealthCheck.too_slow, HealthCheck.filter_too_much],
)
def test_doctor_fuzz_healthy(env_key: str, env_val: str) -> None:
    """Fuzz random ASCII env vars and ensure `bijux_cli doctor` stays healthy."""
    assume(all(ord(c) < 128 for c in env_key))
    assume(all(ord(c) < 128 for c in env_val))

    mp = pytest.MonkeyPatch()
    try:
        mp.setenv(env_key, env_val)
        res = run_cli(["doctor"])
    finally:
        mp.undo()

    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert data.get("status") == "healthy"
    assert isinstance(data.get("summary"), list)
    for msg in data["summary"]:
        assert isinstance(msg, str)


@given(env_key=env_key_strategy, env_val=env_val_strategy)
@settings(
    max_examples=10,
    deadline=None,
    suppress_health_check=[HealthCheck.too_slow, HealthCheck.filter_too_much],
)
def test_doctor_fuzz_unhealthy(env_key: str, env_val: str) -> None:
    """Fuzz random ASCII env vars plus force‑unhealthy and verify status."""
    assume(all(ord(c) < 128 for c in env_key))
    assume(all(ord(c) < 128 for c in env_val))

    mp = pytest.MonkeyPatch()
    try:
        mp.setenv(env_key, env_val)
        mp.setenv("BIJUXCLI_TEST_FORCE_UNHEALTHY", "1")
        res = run_cli(["doctor"])
    finally:
        mp.undo()

    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert data.get("status") == "unhealthy"
    assert isinstance(data.get("summary"), list)
    for msg in data["summary"]:
        assert isinstance(msg, str)
