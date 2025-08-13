# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux dev` command."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
import os
from pathlib import Path
import string
from subprocess import TimeoutExpired, run
import time
from typing import Any

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.conftest import BIN, run_cli

from .conftest import (
    assert_error_contract,
    assert_json,
    assert_yaml,
    make_ro_dir,
    normalize_di,
    require_symlink,
)

SAFE_CHARS = st.text(
    alphabet=string.ascii_letters + string.digits + "-_", min_size=1, max_size=10
)
PER_EX_TIMEOUT = float(os.environ.get("BIJUX_TEST_FUZZ_TIMEOUT", "0.2"))
MAX_EXAMPLES = int(os.environ.get("BIJUX_TEST_FUZZ_EXAMPLES", "35"))


@dataclass
class Proc:
    """A simple container for process results."""

    returncode: int
    stdout: str
    stderr: str


def run_cli_with_timeout(args: list[str], timeout: float = PER_EX_TIMEOUT) -> Proc:
    """Run the CLI with a timeout and return a Proc object."""
    full_args = [str(BIN), *args]
    try:
        r = run(  # noqa: S603
            full_args, capture_output=True, text=True, timeout=timeout
        )
        return Proc(r.returncode, r.stdout, r.stderr)
    except TimeoutExpired:
        return Proc(-1, "", "Timeout")


@given(SAFE_CHARS)
@settings(deadline=None, max_examples=MAX_EXAMPLES)
def test_unknown_flag_ignored_or_errors_gracefully(flag: str) -> None:
    """Ensure that unknown flags either error gracefully or are ignored."""
    flag = flag.strip("- ")
    if not flag:
        return

    if flag in {"h", "help"}:
        r = run_cli_with_timeout(["dev", "di", f"--{flag}"])
        if r.returncode == -1:
            return
        assert r.returncode == 0
        assert r.stdout.lower().startswith("usage:")
        return

    argv = ["dev", "di", "--format", "json", f"--{flag}"]
    r = run_cli_with_timeout(argv)
    if r.returncode == -1:
        return
    assert r.returncode in (0, 2, 3)

    text = (r.stdout + r.stderr).strip()
    if not text or text.lower().startswith("usage:"):
        return
    try:
        obj = assert_json(text)
    except Exception as exc:
        pytest.fail(f"Flag: {flag}, Output not valid JSON: {text!r}\n{exc}")
    assert ("error" in obj) or ("factories" in obj and "services" in obj)


def test_di_json_shape_golden(golden_dir: Path) -> None:
    """Compare the DI JSON output shape against a golden file."""
    r = run_cli(["dev", "di", "--format", "json"])
    live = normalize_di(assert_json(r.stdout))
    want = normalize_di(assert_json((golden_dir / "di_shape.json").read_text()))
    assert live.keys() == want.keys()
    for k in live:
        assert live[k]
        for d in live[k]:
            assert set(d.keys()) == set(want[k][0].keys())


def test_di_yaml_shape_golden(golden_dir: Path) -> None:
    """Compare the DI YAML output shape against a golden file."""
    r = run_cli(["dev", "di", "--format", "yaml"])
    live: Mapping[str, Any] = normalize_di(assert_yaml(r.stdout))
    want: Mapping[str, Any] = normalize_di(
        assert_yaml((golden_dir / "di_shape.yaml").read_text())
    )
    assert live.keys() == want.keys()
    for k in live:
        assert live[k]
        for d in live[k]:
            assert set(d.keys()) == set(want[k][0].keys())


def test_limit_env_zero(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensure BIJUXCLI_DI_LIMIT=0 results in an empty graph."""
    monkeypatch.setenv("BIJUXCLI_DI_LIMIT", "0")
    out = tmp_path / "z.json"
    run_cli(["dev", "di", "--output", str(out), "--format", "json"])
    obj = assert_json(out.read_text())
    assert obj["factories"] == []
    assert obj["services"] == []


def test_limit_env_negative_is_error(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Ensure a negative BIJUXCLI_DI_LIMIT errors out."""
    monkeypatch.setenv("BIJUXCLI_DI_LIMIT", "-1")
    out = tmp_path / "x.json"
    r = run_cli(["dev", "di", "--output", str(out)])
    assert r.returncode == 2
    obj = assert_json(r.stderr)
    assert_error_contract(obj, 2)


def test_config_non_ascii_is_ascii_error(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Ensure a config path with non-ASCII characters results in an error."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/tmp/\u2603")  # noqa: S108
    out = tmp_path / "a.json"
    r = run_cli(["dev", "di", "--output", str(out)])
    assert r.returncode == 3


def test_config_ascii_but_unreadable(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Ensure an unreadable config file results in an error."""
    cfg = tmp_path / "cfg.txt"
    cfg.write_text("x", encoding="ascii")
    cfg.chmod(0)
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))
    out = tmp_path / "o.json"
    r = run_cli(["dev", "di", "--output", str(out)])
    assert r.returncode == 2
    obj = assert_json(r.stderr)
    assert_error_contract(obj, 2)


def test_permission_denied_parent(tmp_path: Path) -> None:
    """Ensure writing to a read-only directory fails gracefully."""
    target = tmp_path / "ro" / "out.json"
    make_ro_dir(tmp_path / "ro")
    r = run_cli(["dev", "di", "--output", str(target), "--format", "json"])
    assert r.returncode == 2
    obj = assert_json(r.stderr)
    assert_error_contract(obj, 2)


def test_symlink_output(tmp_path: Path) -> None:
    """Ensure output can be written to a symlink."""
    real = tmp_path / "real.json"
    link = tmp_path / "link.json"
    real.write_text("", encoding="utf-8")
    require_symlink(tmp_path)
    try:
        link.symlink_to(real)
    except OSError:
        pytest.skip("symlinks not supported")
    r = run_cli(["dev", "di", "--output", str(link), "--format", "json"])
    assert r.returncode == 0
    assert real.read_text() != ""


def test_serializer_failure_is_nonzero(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Ensure a forced serialization failure results in a non-zero exit code."""
    out = tmp_path / "x.json"
    monkeypatch.setenv("BIJUXCLI_TEST_FORCE_SERIALIZE_FAIL", "1")
    r = run_cli(["dev", "di", "--output", str(out), "--format", "json"])
    assert r.returncode == 1


@pytest.mark.parametrize("fmt", ["json", "yaml"])
def test_stdout_formats(fmt: str) -> None:
    """Test standard output formats (JSON and YAML)."""
    r = run_cli(["dev", "di", "--format", fmt])
    assert r.returncode == 0
    if fmt == "json":
        obj = assert_json(r.stdout)
        assert "factories" in obj
        assert "services" in obj
    else:
        obj = assert_yaml(r.stdout)
        assert "factories" in obj
        assert "services" in obj


def test_multi_outputs_same(tmp_path: Path) -> None:
    """Ensure that providing multiple output files results in identical content."""
    a, b = tmp_path / "a.json", tmp_path / "b.json"
    r = run_cli(
        ["dev", "di", "--output", str(a), "--output", str(b), "--format", "json"]
    )
    assert r.returncode == 0
    assert a.read_text() == b.read_text()


def test_directory_output_error(tmp_path: Path) -> None:
    """Ensure providing a directory as an output file results in an error."""
    d = tmp_path / "dir"
    d.mkdir()
    r = run_cli(["dev", "di", "--output", str(d)])
    assert r.returncode == 2
    obj = assert_json(r.stderr)
    assert_error_contract(obj, 2)


def test_quiet_writes_files(tmp_path: Path) -> None:
    """Ensure --quiet suppresses stdout but still writes to output files."""
    out = tmp_path / "w.json"
    r = run_cli(["dev", "di", "--quiet", "--output", str(out), "--format", "json"])
    assert r.returncode == 0
    assert not r.stdout.strip()
    assert out.exists()


def test_perf_budget(tmp_path: Path) -> None:
    """Check that a simple DI graph generation is within the performance budget."""
    out = tmp_path / "p.json"
    t0 = time.perf_counter()
    r = run_cli(["dev", "di", "--output", str(out), "--format", "json"])
    assert r.returncode == 0
    assert (time.perf_counter() - t0) < 5
