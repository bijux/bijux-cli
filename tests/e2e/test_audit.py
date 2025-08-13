# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux audit` command."""

from __future__ import annotations

import difflib
import json
from pathlib import Path
import signal
from subprocess import PIPE, Popen
import sys
import threading
import time
from typing import Any, cast

from hypothesis import HealthCheck, given, settings
import hypothesis.strategies as st
import pytest
import yaml

from tests.e2e.conftest import run_cli

GLOBAL_FLAGS = [
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
    "--pretty",
]

FORMATS = ["json", "yaml", "bogus", "garbage", ""]


def _assert_json(text: str) -> Any:
    """Assert that text is valid JSON and return the parsed object."""
    try:
        return json.loads(text)
    except Exception as e:
        pytest.fail(f"Not valid JSON: {e}\n{text}")


def _assert_yaml(text: str) -> Any:
    """Assert that text is valid YAML and return the parsed object."""
    try:
        return yaml.safe_load(text)
    except Exception as e:
        pytest.fail(f"Not valid YAML: {e}\n{text}")


def _assert_contract(payload: dict[str, Any]) -> None:
    """Assert that the payload adheres to the basic output contract."""
    assert isinstance(payload, dict)
    assert "status" in payload
    assert payload["status"] in (
        "dry-run",
        "completed",
        "failed",
        "skipped",
    )


def assert_error_contract(payload: dict[str, Any], code: int | None = None) -> None:
    """Assert that the payload adheres to the error output contract."""
    assert isinstance(payload, dict)
    assert "error" in payload
    if code is not None:
        assert "code" in payload
        assert payload["code"] == code


def _assert_no_stacktrace(text: str) -> None:
    """Assert that the output does not contain a stacktrace."""
    s = text.lower()
    if "traceback" in s and "site-packages/coverage" in s:
        return
    assert "traceback" not in s
    assert "typer" not in s
    assert "click" not in s


def _assert_ascii(text: str) -> None:
    """Fail if text contains non-ASCII characters."""
    try:
        text.encode("ascii")
    except UnicodeEncodeError:
        pytest.fail("Non-ASCII detected in output/args/env")


@st.composite
def audit_flag_permutations(draw: Any) -> list[str]:
    """Generate permutations of audit command flags for hypothesis."""
    flags = draw(
        st.lists(st.sampled_from(ALL_FLAGS), min_size=0, max_size=6, unique=False)
    )
    if any(f in flags for f in ("--format", "-f")):
        try:
            idx = flags.index("--format")
        except ValueError:
            idx = flags.index("-f")
        flags = flags[: idx + 1] + [draw(st.sampled_from(FORMATS))] + flags[idx + 1 :]
    return cast(list[str], flags)


@pytest.mark.parametrize(("flags", "expected"), GLOBAL_FLAGS)
def test_audit_accepts_all_global_flags(
    tmp_path: Path, flags: list[str], expected: int
) -> None:
    """Test that the audit command accepts all global flags without crashing."""
    out_file = tmp_path / "audit.json"
    res = run_cli(["audit", "--dry-run", "--output", str(out_file), *flags])
    assert res.returncode == expected
    _assert_no_stacktrace(res.stdout + res.stderr)


def test_audit_default_json_contract(tmp_path: Path) -> None:
    """Test that the default output is valid JSON adhering to the contract."""
    out_file = tmp_path / "audit.json"
    res = run_cli(["audit", "--dry-run", "--output", str(out_file)])
    assert res.returncode == 0
    payload = _assert_json(out_file.read_text())
    _assert_contract(payload)
    _assert_no_stacktrace(out_file.read_text())


def test_audit_yaml_contract(tmp_path: Path) -> None:
    """Test that YAML output is valid and adheres to the contract."""
    out_file = tmp_path / "audit.yaml"
    res = run_cli(["audit", "--dry-run", "--output", str(out_file), "--format", "yaml"])
    assert res.returncode == 0
    payload = _assert_yaml(out_file.read_text())
    _assert_contract(payload)
    _assert_no_stacktrace(out_file.read_text())


def test_audit_idempotency(tmp_path: Path) -> None:
    """Test that repeated runs of the audit command produce consistent output."""
    out_file = tmp_path / "audit.json"
    run_cli(["audit", "--dry-run", "--output", str(out_file)])
    p1 = _assert_json(out_file.read_text())
    run_cli(["audit", "--dry-run", "--output", str(out_file)])
    p2 = _assert_json(out_file.read_text())
    assert p1["status"] == p2["status"]


def test_audit_format_flag_duplicate_and_case(tmp_path: Path) -> None:
    """Test that the --format flag is case-insensitive and the last one wins."""
    out_file = tmp_path / "audit.file"
    run_cli(
        [
            "audit",
            "--dry-run",
            "--output",
            str(out_file),
            "--format",
            "yaml",
            "--format",
            "json",
        ]
    )
    p1 = _assert_json(out_file.read_text())
    _assert_contract(p1)
    run_cli(["audit", "--dry-run", "--output", str(out_file), "--format", "YAML"])
    p2 = _assert_yaml(out_file.read_text())
    _assert_contract(p2)


def test_audit_quiet_suppresses_output(tmp_path: Path) -> None:
    """Test that the --quiet flag suppresses all stdout and stderr."""
    out_file = tmp_path / "audit.json"
    for flag in ("-q", "--quiet"):
        res = run_cli(["audit", "--dry-run", "--output", str(out_file), flag])
        assert res.returncode == 0
        assert not res.stdout.strip()
        assert not res.stderr.strip()


def test_audit_precedence_rules(tmp_path: Path) -> None:
    """Test the precedence of formatting and verbosity flags."""
    out_file = tmp_path / "audit.json"
    res = run_cli(
        ["audit", "--dry-run", "--output", str(out_file), "--quiet", "--debug"]
    )
    assert res.returncode == 0
    assert not res.stdout.strip()
    assert not res.stderr.strip()
    res2 = run_cli(
        ["audit", "--dry-run", "--output", str(out_file), "--debug", "--no-pretty"]
    )
    assert res2.returncode == 0
    lines = out_file.read_text().splitlines()
    assert len(lines) > 1


def test_audit_structured_error_invalid_format() -> None:
    """Test that an invalid format produces a structured error."""
    res = run_cli(["audit", "--dry-run", "--format", "bogus"])
    assert res.returncode == 2
    payload = _assert_json(res.stderr)
    assert_error_contract(payload, code=2)


def test_audit_structured_error_unknown_flag() -> None:
    """Test that an unknown flag produces a structured error."""
    res = run_cli(["audit", "--dry-run", "--notaflag"])
    assert res.returncode == 2
    payload = _assert_json(res.stdout)
    assert_error_contract(payload, code=2)


def test_audit_ascii_hygiene(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that output is always ASCII and non-ASCII env vars cause an error."""
    res = run_cli(["audit", "--dry-run"])
    _assert_ascii(res.stdout + res.stderr)
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/tmp/\u2603")  # noqa: S108
    res2 = run_cli(["audit", "--dry-run"])
    assert res2.returncode == 3
    payload = _assert_json(res2.stderr)
    assert_error_contract(payload, code=3)


def test_audit_no_config_env_leak(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that sensitive config paths are not leaked in the output."""
    out_file = tmp_path / "audit.json"
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/my/path/config.env")
    res = run_cli(["audit", "--dry-run", "--output", str(out_file)])
    assert res.returncode in (0, 1, 2)
    stdout = res.stdout.lower()
    stderr = res.stderr.lower()
    assert "config.env" not in stdout
    assert "config.env" not in stderr


def test_audit_debug_and_verbose_add_runtime(tmp_path: Path) -> None:
    """Test that debug/verbose flags add runtime info to the output."""
    out_file = tmp_path / "audit.json"
    for flag in ("-v", "--debug", "-d"):
        run_cli(["audit", "--dry-run", "--output", str(out_file), flag])
        payload = _assert_json(out_file.read_text())
        assert "python" in payload
        assert "platform" in payload


def test_audit_pretty_and_no_pretty(tmp_path: Path) -> None:
    """Test that --pretty and --no-pretty flags control output formatting."""
    out_file = tmp_path / "pretty.json"
    run_cli(["audit", "--dry-run", "--output", str(out_file), "--pretty"])
    lines = out_file.read_text().splitlines()
    assert len(lines) > 1
    run_cli(["audit", "--dry-run", "--output", str(out_file), "--no-pretty"])
    lines2 = out_file.read_text().splitlines()
    assert len(lines2) == 1


def test_audit_help_contract() -> None:
    """Test that the --help output adheres to the required contract."""
    res = run_cli(["audit", "--help"])
    assert res.returncode == 0
    lines = res.stdout.strip().splitlines()
    assert lines[0].lower().startswith("usage:")
    assert len(lines) <= 50
    help_text = "\n".join(lines).lower()
    required_flags = ["-q", "--quiet", "-f", "--format", "--pretty", "-d", "--debug"]
    for flag in required_flags:
        assert flag in help_text, f"missing flag in help output: {flag}"
    _assert_no_stacktrace(res.stdout)


def test_command_performance_audit(tmp_path: Path) -> None:
    """Test that the audit command executes within a reasonable time."""
    out_file = tmp_path / "audit.json"
    t0 = time.monotonic()
    res = run_cli(["audit", "--dry-run", "--output", str(out_file)])
    assert (time.monotonic() - t0) < 5.0
    assert res.returncode == 0


def test_audit_idempotency_multiple_runs(tmp_path: Path) -> None:
    """Test that repeated runs are idempotent."""
    out_file = tmp_path / "audit.json"
    vals = []
    for _ in range(3):
        run_cli(["audit", "--dry-run", "--output", str(out_file)])
        vals.append(_assert_json(out_file.read_text())["status"])
    assert all(x == vals[0] for x in vals)


def test_audit_concurrent_invocations(tmp_path: Path) -> None:
    """Test that concurrent audit commands do not interfere with each other."""
    out_file = tmp_path / "audit.json"
    results = []

    def runit() -> None:
        """Helper to run audit and collect results."""
        run_cli(["audit", "--dry-run", "--output", str(out_file)])
        results.append(_assert_json(out_file.read_text()))

    threads = [threading.Thread(target=runit) for _ in range(3)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    for payload in results:
        _assert_contract(payload)


def test_audit_exit_codes(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the command returns the correct exit codes for different scenarios."""
    out_file = tmp_path / "audit.json"
    res1 = run_cli(["audit", "--dry-run", "--output", str(out_file)])
    assert res1.returncode == 0
    res2 = run_cli(["audit", "--dry-run", "--format", "bogus"])
    assert res2.returncode == 2
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/tmp/\u2603")  # noqa: S108
    res3 = run_cli(["audit", "--dry-run"])
    assert res3.returncode == 3


def test_audit_output_file_overwrite(tmp_path: Path) -> None:
    """Test that using --output overwrites, not appends to, an existing file."""
    out_file = tmp_path / "audit.json"
    out_file.write_text('{"foo": "bar"}\n')
    res = run_cli(["audit", "--dry-run", "--output", str(out_file)])
    assert res.returncode == 0
    payload = _assert_json(out_file.read_text())
    _assert_contract(payload)
    assert "foo" not in payload


def test_audit_unknown_extra_args_errors(tmp_path: Path) -> None:
    """Test that extraneous arguments cause a structured error."""
    res = run_cli(["audit", "--dry-run", "extraneous"])
    assert res.returncode != 0
    try:
        payload = _assert_json(res.stdout)
        assert_error_contract(payload, code=2)
    except Exception:
        assert "error" in (res.stdout + res.stderr).lower()


def test_audit_too_many_positional_errors() -> None:
    """Test that multiple extraneous arguments cause an error."""
    res = run_cli(["audit", "too", "many", "args"])
    assert res.returncode != 0


def test_audit_malformed_config_state(tmp_path: Path) -> None:
    """Test a graceful failure with a malformed config file."""
    conf = tmp_path / "bad.env"
    conf.write_text("not=valid:env::\n")
    env = {"BIJUXCLI_CONFIG": str(conf)}
    res = run_cli(["audit", "--dry-run"], env=env)
    assert res.returncode != 0 or "error" in (res.stdout + res.stderr).lower()


def test_audit_signal_interrupt(tmp_path: Path) -> None:
    """Test that SIGINT interrupts the command gracefully."""
    out_file = tmp_path / "audit.json"
    proc = Popen(  # noqa: S603
        [
            sys.executable,
            "-m",
            "bijux_cli",
            "audit",
            "--dry-run",
            "--output",
            str(out_file),
        ],
        stdout=PIPE,
        stderr=PIPE,
        text=True,
    )
    time.sleep(0.2)
    proc.send_signal(signal.SIGINT)
    stdout, stderr = proc.communicate(timeout=2)
    assert proc.returncode != 0
    msg = (stdout + stderr).lower()
    assert "interrupt" in msg or "signal" in msg or proc.returncode == -signal.SIGINT


@given(flags=audit_flag_permutations())
@settings(max_examples=20, deadline=None, suppress_health_check=[HealthCheck.too_slow])
def test_audit_hypothesis_flags(flags: list[str]) -> None:
    """Test various flag combinations generated by hypothesis."""
    for i, flag in enumerate(flags):
        if (
            flag in ("--format", "-f")
            and i + 1 < len(flags)
            and flags[i + 1].startswith("-")
        ):
            return

    res = run_cli(["audit", "--dry-run", *flags])

    if "--help" in flags or "-h" in flags:
        assert res.returncode == 0
        assert res.stdout.lstrip().lower().startswith("usage:")
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

    assert res.returncode == 0
    assert res.stdout.strip(), "No output when expected"
    out = res.stdout.strip()
    try:
        data = json.loads(out)
    except Exception:
        try:
            data = yaml.safe_load(out)
        except Exception:
            pytest.fail(f"Output not valid JSON or YAML: {out!r}")
    assert isinstance(data, dict)
    assert "status" in data


def test_command_performance_audit_under_load(tmp_path: Path) -> None:
    """Test that repeated audit calls are consistently fast."""
    times = []
    for _ in range(10):
        t0 = time.perf_counter()
        res = run_cli(["audit", "--dry-run"])
        t1 = time.perf_counter()
        assert res.returncode == 0
        assert "status" in json.loads(res.stdout)
        times.append(t1 - t0)
    max_time = max(times)
    assert max_time < 5.0, f"Max single audit call too slow: {max_time:.2f}s"


def test_audit_golden_output(tmp_path: Path) -> None:
    """Golden test: output matches expected fixture."""
    out_file = tmp_path / "audit.json"
    run_cli(["audit", "--dry-run", "--output", str(out_file)])
    golden = Path("tests/e2e/test_fixtures/audit/audit.json").read_text()
    actual = out_file.read_text()
    diff = list(difflib.unified_diff(golden.splitlines(), actual.splitlines()))
    assert not diff, "\n".join(diff)


def test_audit_stdout_default_json(tmp_path: Path) -> None:
    """If no --output is given, audit writes JSON to stdout by default."""
    res = run_cli(["audit", "--dry-run"])
    assert res.returncode == 0
    payload = json.loads(res.stdout)
    assert "status" in payload
    assert payload["status"] in (
        "dry-run",
        "completed",
        "skipped",
        "failed",
    )
    assert not res.stderr.strip(), "No stderr on success"
    assert not (tmp_path / "audit.json").exists()


@pytest.mark.parametrize(
    ("fmt", "loader"),
    [
        ("json", json.loads),
        ("yaml", yaml.safe_load),
    ],
)
def test_audit_stdout_default_format_variants(fmt: str, loader: Any) -> None:
    """audit --dry-run -f <fmt> without --output writes to stdout in that format."""
    args = ["audit", "--dry-run", "-f", fmt]
    res = run_cli(args)
    assert res.returncode == 0
    payload = loader(res.stdout)
    assert isinstance(payload, dict)
    assert "status" in payload
    assert payload["status"] in (
        "dry-run",
        "completed",
        "skipped",
        "failed",
    )
    assert "traceback" not in res.stdout.lower()


def test_audit_invalid_output_path_errors(tmp_path: Path) -> None:
    """Writing to a non-existent directory should produce a structured error."""
    bad = tmp_path / "no_such_dir" / "out.json"
    res = run_cli(["audit", "--dry-run", "--output", str(bad)])
    assert res.returncode == 2
    payload = json.loads(res.stdout or res.stderr)
    assert "error" in payload
    assert payload["code"] == 2


def test_audit_directory_as_output(tmp_path: Path) -> None:
    """Pointing --output at an existing directory produces a structured error."""
    d = tmp_path / "adir"
    d.mkdir()
    res = run_cli(["audit", "--dry-run", "--output", str(d)])
    assert res.returncode == 2
    payload = json.loads(res.stderr)
    assert "error" in payload
    assert payload["code"] == 2


def test_audit_symlink_and_dotfile_paths(tmp_path: Path) -> None:
    """audit should handle symlinks and dotfile output paths the same as normal files."""
    real = tmp_path / "real.json"
    real.write_text('{"foo":"bar"}')
    link = tmp_path / "link.json"
    link.symlink_to(real)
    res1 = run_cli(["audit", "--dry-run", "--output", str(link)])
    assert res1.returncode == 0

    dot = tmp_path / ".audit.json"
    res2 = run_cli(["audit", "--dry-run", "--output", str(dot)])
    assert res2.returncode == 0
    assert dot.exists()
    data = json.loads(dot.read_text())
    assert data["status"] == "dry-run"


def test_audit_quiet_no_output_to_stdout(tmp_path: Path) -> None:
    """--quiet with no --output should suppress stdout/stderr and still exit 0."""
    res = run_cli(["audit", "--dry-run", "--quiet"])
    assert res.returncode == 0
    assert not res.stdout.strip()
    assert not res.stderr.strip()


@pytest.mark.parametrize(
    ("flag", "value"),
    [
        (["-f", "json"], json.loads),
        (["-f", "JSON"], json.loads),
        (["-f", "yaml"], yaml.safe_load),
        (["-f", "YAML"], yaml.safe_load),
    ],
)
def test_audit_short_format_flag_cases(
    tmp_path: Path, flag: list[str], value: Any
) -> None:
    """Test that short -f is case-insensitive and last-one-wins."""
    out = tmp_path / "audit.file"
    args = ["audit", "--dry-run", "--output", str(out), "-f", "bogus"] + flag
    res = run_cli(args)
    assert res.returncode == 0
    data = value(out.read_text())
    assert isinstance(data, dict)
    assert "status" in data


def test_audit_quiet_does_not_suppress_validation_error_code() -> None:
    """ADR Test: --quiet must not suppress the exit code from a validation error."""
    res = run_cli(["audit", "--dry-run", "--quiet", "extraneous"])

    assert res.returncode == 2, (
        "ADR Violation: --quiet suppressed a validation error exit code."
    )
    assert not res.stdout.strip(), "No stdout on --quiet"
    assert not res.stderr.strip(), "No stderr on --quiet"


def test_audit_help_precedes_di_failure(monkeypatch: pytest.MonkeyPatch) -> None:
    """ADR Test: --help must short-circuit everything, even a DI container failure."""

    def mock_resolve_fails(self: Any, protocol: Any) -> None:
        """Simulate a DI container crash."""
        raise RuntimeError("Simulated DI Container Crash")

    from bijux_cli.core.di import DIContainer

    monkeypatch.setattr(DIContainer, "resolve", mock_resolve_fails)

    res = run_cli(["audit", "--help"])

    assert res.returncode == 0, (
        "ADR Violation: --help did not short-circuit a DI failure."
    )
    assert "usage: bijux audit" in res.stdout.lower()
    assert not res.stderr.strip()


def test_audit_verbose_info_persists_with_output_flag(tmp_path: Path) -> None:
    """ADR Test: --verbose info must be in stdout payload when --output is used."""
    out_file = tmp_path / "audit.json"

    res = run_cli(["audit", "--dry-run", "--output", str(out_file), "--verbose"])

    assert res.returncode == 0

    stdout_payload = _assert_json(res.stdout)
    assert stdout_payload["status"] == "written"

    assert "python" in stdout_payload, (
        "ADR Violation: Runtime info missing from final stdout payload with --output."
    )
    assert "platform" in stdout_payload, (
        "ADR Violation: Runtime info missing from final stdout payload with --output."
    )

    file_payload = _assert_json(out_file.read_text())
    assert "python" in file_payload
    assert "platform" in file_payload


def test_audit_help_precedes_and_ignores_other_flag_errors() -> None:
    """ADR Test: --help must short-circuit and ignore errors from other invalid flags."""
    res = run_cli(["audit", "--help", "--format"])

    assert res.returncode == 0, (
        "ADR Violation: --help did not short-circuit an invalid flag error."
    )
    assert "usage: bijux audit" in res.stdout.lower()
    assert not res.stderr.strip()

    res_2 = run_cli(["audit", "-h", "--unsupported-flag"])
    assert res_2.returncode == 0, (
        "ADR Violation: --help did not short-circuit an unknown flag error."
    )
    assert "usage: bijux audit" in res_2.stdout.lower()
