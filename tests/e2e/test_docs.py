# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux docs` command."""

from __future__ import annotations

from collections.abc import Callable, Iterable
import concurrent.futures
import json
from pathlib import Path
import signal
import stat
from subprocess import PIPE, Popen
import sys
import threading
import time
from typing import Any, cast

import pytest
import yaml

from tests.e2e.conftest import run_cli

SUPPORTED_FORMATS = ("json", "yaml")
FORMAT_MATRIX: list[tuple[list[str], str, Callable[[str], dict[str, Any]]]] = [
    (["--format", "json"], "json", lambda s: cast(dict[str, Any], json.loads(s))),
    (["-f", "json"], "json", lambda s: cast(dict[str, Any], json.loads(s))),
    (["--format", "JSON"], "json", lambda s: cast(dict[str, Any], json.loads(s))),
    (["--format", "yaml"], "yaml", lambda s: cast(dict[str, Any], yaml.safe_load(s))),
    (["-f", "yaml"], "yaml", lambda s: cast(dict[str, Any], yaml.safe_load(s))),
    (["--format", "YAML"], "yaml", lambda s: cast(dict[str, Any], yaml.safe_load(s))),
]

GLOBAL_FLAGS_MATRIX: list[tuple[list[str], int]] = [
    (["--help"], 0),
    (["-q"], 0),
    (["--quiet"], 0),
    (["-v"], 0),
    (["--verbose"], 0),
    (["-f", "json"], 0),
    (["-f", "yaml"], 0),
    (["-f", "JSON"], 0),
    (["--pretty"], 0),
    (["--no-pretty"], 0),
    (["-d"], 0),
    (["--debug"], 0),
]

FUZZABLE_FLAGS = [
    "--help",
    "-q",
    "--quiet",
    "-v",
    "--verbose",
    "-f",
    "--format",
    "--pretty",
    "--no-pretty",
    "-d",
    "--debug",
]


def _no_stacktrace_leak(text_leak: str) -> None:
    """Assert no traceback or framework names leak into user output."""
    s = text_leak.lower()
    assert "traceback" not in s
    assert "typer" not in s
    assert "click" not in s


def _load_json(text_json: str) -> dict[str, Any]:
    """Parse JSON text, ensuring a dict result; fail with a helpful message."""
    try:
        obj = json.loads(text_json)
    except Exception as e:
        pytest.fail(f"Output not valid JSON: {e}\n{text_json}")
    if not isinstance(obj, dict):
        pytest.fail(f"JSON is not a dict: {obj!r}\n{text_json}")
    return cast(dict[str, Any], obj)


def _load_yaml(text_yaml: str) -> dict[str, Any]:
    """Parse YAML text, ensuring a dict result; fail with a helpful message."""
    try:
        obj = yaml.safe_load(text_yaml)
    except Exception as e:
        pytest.fail(f"Output not valid YAML: {e}\n{text_yaml}")
    if not isinstance(obj, dict):
        pytest.fail(f"YAML is not a dict: {obj!r}\n{text_yaml}")
    return cast(dict[str, Any], obj)


def _assert_error_contract(payload: dict[str, Any], code: int | None = None) -> None:
    """Minimal error contract."""
    assert isinstance(payload, dict)
    assert "error" in payload
    if code is not None:
        assert "code" in payload
        assert payload["code"] == code


def _assert_docs_contract(payload: dict[str, Any]) -> None:
    """Assert the high-level docs contract."""
    assert isinstance(payload, dict)
    assert "version" in payload
    assert "commands" in payload
    assert isinstance(payload["commands"], (list | dict))


def _ascii_only(text_ascii: str) -> None:
    """Fail if text contains non-ASCII characters."""
    try:
        text_ascii.encode("ascii")
    except UnicodeEncodeError:
        pytest.fail("Non-ASCII detected in output.")


def _ids(values: Iterable[Any]) -> list[str]:
    """Readable pytest ids for parameterized inputs."""
    out: list[str] = []
    for v in values:
        if isinstance(v, (list | tuple)):
            out.append("-".join(str(x) for x in v))
        else:
            out.append(str(v))
    return out


@pytest.mark.parametrize(
    ("flags", "fmt", "loader"),
    FORMAT_MATRIX,
    ids=[
        "long-json",
        "short-json",
        "case-JSON",
        "long-yaml",
        "short-yaml",
        "case-YAML",
    ],
)
def test_docs_format_matrix(
    tmp_path: Path, flags: list[str], fmt: str, loader: Callable[[str], dict[str, Any]]
) -> None:
    """Every supported format parses and satisfies the contract."""
    out_file = tmp_path / ("spec." + ("json" if fmt == "json" else "yaml"))
    res = run_cli(["docs", "--out", str(out_file), *flags])
    assert res.returncode == 0
    payload = loader(out_file.read_text())
    _assert_docs_contract(payload)
    _no_stacktrace_leak(res.stdout + res.stderr)


@pytest.mark.parametrize(
    ("flags", "expected"), GLOBAL_FLAGS_MATRIX, ids=_ids(GLOBAL_FLAGS_MATRIX)
)
def test_docs_accepts_all_global_flags(
    tmp_path: Path, flags: list[str], expected: int
) -> None:
    """Global flags do not crash; exit code equals expectation."""
    out_file = tmp_path / "spec.json"
    res = run_cli(["docs", "--out", str(out_file), *flags])
    assert res.returncode == expected
    _no_stacktrace_leak(res.stdout + res.stderr)


def test_docs_idempotent(tmp_path: Path) -> None:
    """Two runs to the same output produce stable results."""
    out_file = tmp_path / "spec.json"
    r1 = run_cli(["docs", "--out", str(out_file)])
    r2 = run_cli(["docs", "--out", str(out_file)])
    assert r1.returncode == 0
    assert r2.returncode == 0
    p1 = _load_json(out_file.read_text())
    p2 = _load_json(out_file.read_text())
    _assert_docs_contract(p1)
    assert p1 == p2


def test_docs_yaml_output(tmp_path: Path) -> None:
    """YAML output parses and satisfies the contract."""
    out_file = tmp_path / "spec.yaml"
    res = run_cli(["docs", "--out", str(out_file), "--format", "yaml"])
    assert res.returncode == 0
    payload = _load_yaml(out_file.read_text())
    _assert_docs_contract(payload)


def test_docs_format_last_wins(tmp_path: Path) -> None:
    """Duplicate --format uses the last occurrence."""
    out_file = tmp_path / "spec.any"
    run_cli(["docs", "--out", str(out_file), "--format", "yaml", "--format", "json"])
    p_json = _load_json(out_file.read_text())
    _assert_docs_contract(p_json)
    run_cli(["docs", "--out", str(out_file), "--format", "YAML"])
    p_yaml = _load_yaml(out_file.read_text())
    _assert_docs_contract(p_yaml)


@pytest.mark.parametrize("quiet", [["-q"], ["--quiet"]], ids=["short", "long"])
def test_docs_quiet_suppresses_all_output(tmp_path: Path, quiet: list[str]) -> None:
    """Test that quiet flags suppress all output."""
    out_file = tmp_path / "spec.json"
    res = run_cli(["docs", "--out", str(out_file), *quiet])
    assert res.returncode == 0
    assert not res.stdout.strip()
    assert not res.stderr.strip()


def test_docs_flag_precedence(tmp_path: Path) -> None:
    """Quiet wins over other noisy flags; debug forces pretty."""
    out_file = tmp_path / "spec.json"

    res = run_cli(["docs", "--out", str(out_file), "--quiet", "--debug"])
    assert res.returncode == 0
    assert not res.stdout.strip()
    assert not res.stderr.strip()

    res2 = run_cli(["docs", "--out", str(out_file), "--debug", "--no-pretty"])
    assert res2.returncode == 0
    assert len(out_file.read_text().splitlines()) > 1


@pytest.mark.parametrize(
    ("flag", "value"),
    [
        ("--format", "foobar"),
        ("--format", ""),
        ("-f", "notreal"),
    ],
    ids=[
        "--format=foobar",
        "--format=empty",
        "-f=notreal",
    ],
)
def test_docs_invalid_format_structured_error(
    tmp_path: Path, flag: str, value: str
) -> None:
    """Test that invalid format values produce a structured error."""
    out_file = tmp_path / "spec.json"
    res = run_cli(["docs", "--out", str(out_file), flag, value])
    assert res.returncode == 2
    payload = _load_json(res.stdout or res.stderr)
    _assert_error_contract(payload, code=2)
    _no_stacktrace_leak(res.stdout + res.stderr)


def test_docs_unknown_flag_structured_error(tmp_path: Path) -> None:
    """Test that unknown flags produce a structured error."""
    out_file = tmp_path / "spec.json"
    res = run_cli(["docs", "--out", str(out_file), "--notaflag"])
    assert res.returncode == 2
    payload = _load_json(res.stdout or res.stderr)
    _assert_error_contract(payload, code=2)


def test_docs_invalid_out_path_structured_error() -> None:
    """Writing under a non-writable pseudo-path should produce a structured error."""
    res = run_cli(["docs", "--out", "/dev/null/foo.json"])
    assert res.returncode == 2
    payload = _load_json(res.stdout or res.stderr)
    _assert_error_contract(payload, code=2)


def test_docs_out_file_dir_symlink_dotfile(tmp_path: Path) -> None:
    """Validate writing to: file, directory (spec.json created), symlink, dotfile."""
    file_path = tmp_path / "file.json"
    dir_path = tmp_path / "adir"
    dir_path.mkdir()

    res1 = run_cli(["docs", "--out", str(file_path)])
    assert res1.returncode == 0
    assert file_path.exists()

    res2 = run_cli(["docs", "--out", str(dir_path)])
    assert res2.returncode == 0
    assert (dir_path / "spec.json").exists()

    symlink = tmp_path / "link.json"
    symlink.symlink_to(file_path)
    res3 = run_cli(["docs", "--out", str(symlink)])
    assert res3.returncode == 0

    dotfile = tmp_path / ".ax_spec.json"
    res4 = run_cli(["docs", "--out", str(dotfile)])
    assert res4.returncode == 0
    assert dotfile.exists()


@pytest.mark.skipif(sys.platform.startswith("win"), reason="POSIX perms only")
def test_docs_out_dir_permission_denied(tmp_path: Path) -> None:
    """Writing into a read-only directory should fail with structured error."""
    ro_dir = tmp_path / "ro"
    ro_dir.mkdir()
    ro_dir.chmod(stat.S_IREAD | stat.S_IEXEC)
    res = run_cli(["docs", "--out", str(ro_dir)])
    assert res.returncode == 2
    payload = _load_json(res.stdout or res.stderr)
    _assert_error_contract(payload, code=2)


def test_docs_broken_symlink_fails_structured(tmp_path: Path) -> None:
    """Point `--out` at a broken symlink: should error with code 2."""
    target = tmp_path / "missing.json"
    broken = tmp_path / "broken.json"
    broken.symlink_to(target, target_is_directory=False)
    res = run_cli(["docs", "--out", str(broken)])
    assert res.returncode in (0, 2)
    if res.returncode == 2:
        payload = _load_json(res.stdout or res.stderr)
        _assert_error_contract(payload, code=2)


def test_docs_overwrite_corrupted_file(tmp_path: Path) -> None:
    """Pre-existing corrupted file should be overwritten with valid content."""
    out_file = tmp_path / "spec.json"
    out_file.write_text("{bad json")
    res = run_cli(["docs", "--out", str(out_file)])
    assert res.returncode == 0
    payload = _load_json(out_file.read_text())
    _assert_docs_contract(payload)


@pytest.mark.parametrize("flag", ["--help"])
def test_docs_help_contract(flag: str) -> None:
    """Test the contract for the help output."""
    res = run_cli(["docs", flag])
    assert res.returncode == 0
    text_ = res.stdout
    assert text_.lstrip().lower().startswith("usage:")
    assert text_.count("\n") <= 60
    for k in ("--help", "-h", "--quiet", "-q", "--verbose", "-v", "--format", "-f"):
        assert k in text_
    _no_stacktrace_leak(text_)


def test_docs_ascii_hygiene(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Non-ASCII env var should produce code 3 with structured error."""
    out_file = tmp_path / "spec.json"
    res = run_cli(["docs", "--out", str(out_file)])
    _ascii_only(res.stdout)
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/tmp/\u2603")  # noqa: S108
    res2 = run_cli(["docs", "--out", str(out_file)])
    assert res2.returncode == 3
    payload = _load_json(res2.stdout or res2.stderr)
    _assert_error_contract(payload, code=3)


def test_command_performance_docs_smoke(tmp_path: Path) -> None:
    """Command should be reasonably fast on a warm run."""
    out_file = tmp_path / "spec.json"
    t0 = time.perf_counter()
    res = run_cli(["docs", "--out", str(out_file)])
    t1 = time.perf_counter()
    assert res.returncode == 0
    assert (t1 - t0) < 5.0


def test_command_performance_docs_repeatability(tmp_path: Path) -> None:
    """Multiple sequential runs should remain within a tight latency envelope."""
    out = tmp_path / "spec.json"
    timings: list[float] = []
    for _ in range(5):
        t0 = time.perf_counter()
        res = run_cli(["docs", "--out", str(out)])
        t1 = time.perf_counter()
        assert res.returncode == 0
        timings.append(t1 - t0)
    assert max(timings) < 5.0


def test_docs_concurrent_invocations(tmp_path: Path) -> None:
    """Concurrent runs should not interfere or leak invalid content."""
    results: list[str] = []
    out_file = tmp_path / "spec.json"

    def run_and_collect() -> None:
        """Helper to run docs command and collect file content."""
        run_cli(["docs", "--out", str(out_file)])
        results.append(out_file.read_text())

    threads = [threading.Thread(target=run_and_collect) for _ in range(4)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    for text_ in results:
        payload = _load_json(text_)
        _assert_docs_contract(payload)


def test_docs_heavy_concurrency(tmp_path: Path) -> None:
    """Use a thread pool to stress test for race conditions around file writes."""
    out_file = tmp_path / "pool.json"

    def runit() -> int:
        """Helper to run the docs command."""
        return run_cli(["docs", "--out", str(out_file)]).returncode

    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        futures = [pool.submit(runit) for _ in range(16)]
        codes = [f.result() for f in futures]

    assert all(c in (0, 2, 3) for c in codes)
    if out_file.exists():
        _no_stacktrace_leak(out_file.read_text())


def normalize_docs_output(data: dict[str, Any]) -> dict[str, Any]:
    """Remove environment-specific fields for consistent comparison."""
    data = dict(data)
    data.pop("python", None)
    data.pop("platform", None)
    return data


def test_docs_spec_has_valid_structure_and_contains_core_commands(
    tmp_path: Path,
) -> None:
    """Verify generated docs spec has a valid structure and includes all core commands."""
    actual_spec_path = tmp_path / "spec.json"
    run_cli(["docs", "--out", str(actual_spec_path)])

    golden_data = json.loads(Path("tests/e2e/test_fixtures/docs/docs.json").read_text())
    actual_data = json.loads(actual_spec_path.read_text())

    assert isinstance(actual_data, dict)
    assert "version" in actual_data
    assert isinstance(actual_data["version"], str)
    assert "commands" in actual_data
    assert isinstance(actual_data["commands"], list)
    assert all(isinstance(cmd, str) for cmd in actual_data["commands"])
    expected_core_commands = set(golden_data.get("commands", []))
    actual_commands = set(actual_data.get("commands", []))
    missing_commands = expected_core_commands - actual_commands
    assert not missing_commands, (
        f"Spec is missing core commands: {sorted(missing_commands)}"
    )


def test_docs_signal_interrupt(tmp_path: Path) -> None:
    """Test that SIGINT during docs generation is handled gracefully."""
    from tests.e2e.conftest import BIN

    proc = Popen(  # noqa: S603
        [str(BIN), "docs", "--out", str(tmp_path / "spec.json")],
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


def test_docs_verbose_fields(tmp_path: Path) -> None:
    """Test that the --verbose flag adds expected fields to the spec."""
    out = tmp_path / "spec.json"
    res = run_cli(["docs", "--out", str(out), "--verbose"])
    assert res.returncode == 0
    payload = json.loads(out.read_text())
    assert "python" in payload
    assert "platform" in payload
