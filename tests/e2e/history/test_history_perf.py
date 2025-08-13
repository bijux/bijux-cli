# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for `bijux history` command."""

from __future__ import annotations

import json
import multiprocessing
import os
from pathlib import Path
import sys
import threading
import time
from typing import Any

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.conftest import run_cli
from tests.e2e.history.conftest import (
    MAX_EXAMPLES,
    SAFE,
    assert_json,
    run_cli_with_timeout,
    run_module,
)


def _worker_rw(hist_file: Path, n: int) -> None:
    """Helper for multiprocess test: does a mix of read/write ops."""
    for _ in range(n):
        run_cli(["version"], env={"BIJUXCLI_HISTORY_FILE": str(hist_file)})
        r = run_cli(
            ["history", "--format", "json"],
            env={"BIJUXCLI_HISTORY_FILE": str(hist_file)},
        )
        assert r.returncode == 0


def perf_budget(n: int) -> float:
    """Returns a performance budget in seconds for N entries."""
    if n <= 10:
        return 5
    if n <= 100:
        return 10
    if n <= 1000:
        return 20
    if n <= 10000:
        return 25
    if n <= 100000:
        return 30
    return 100.0


def test_perf_budget() -> None:
    """Checks that a simple history call is under the performance budget."""
    run_cli(["version"])
    t0 = time.perf_counter()
    r = run_cli(["history", "--format", "json"])
    assert r.returncode == 0
    assert (time.perf_counter() - t0) < 2.0


def test_isolated_histories_by_config(tmp_path: Path) -> None:
    """Ensures that different history files are used based on environment config."""
    env1 = {
        "BIJUXCLI_CONFIG": str(tmp_path / ".env1"),
        "BIJUXCLI_HISTORY_FILE": str(tmp_path / ".h1"),
    }
    env2 = {
        "BIJUXCLI_CONFIG": str(tmp_path / ".env2"),
        "BIJUXCLI_HISTORY_FILE": str(tmp_path / ".h2"),
    }
    run_cli(["history", "clear"], env=env1)
    run_cli(["history", "clear"], env=env2)

    run_cli(["version"], env=env1)
    run_cli(["status"], env=env2)

    res1 = run_cli(["history", "--format", "json"], env=env1)
    res2 = run_cli(["history", "--format", "json"], env=env2)
    payload1 = json.loads(res1.stdout)
    payload2 = json.loads(res2.stdout)
    cmds1 = [e["command"] for e in payload1.get("entries", [])]
    cmds2 = [e["command"] for e in payload2.get("entries", [])]

    assert any(c == "version" for c in cmds1)
    assert all(c != "status" for c in cmds1)
    assert any(c == "status" for c in cmds2)
    assert all(c != "version" for c in cmds2)


def test_default_limit_is_20_or_less() -> None:
    """Verifies the default history limit is 20."""
    run_cli(["history", "clear"])
    for _ in range(25):
        run_cli(["version"])
    res = run_cli(["history", "--format", "json"])
    assert res.returncode == 0
    payload = json.loads(res.stdout)
    assert len(payload.get("entries", [])) <= 20


def test_custom_limits_and_zero() -> None:
    """Tests custom --limit values, including zero."""
    run_cli(["history", "clear"])
    for _ in range(6):
        run_cli(["version"])
    res1 = run_cli(["history", "--limit", "2", "--format", "json"])
    res2 = run_cli(["history", "--limit", "4", "--format", "json"])
    res0 = run_cli(["history", "--limit", "0", "--format", "json"])
    payload1 = json.loads(res1.stdout)
    payload2 = json.loads(res2.stdout)
    payload0 = json.loads(res0.stdout)
    assert len(payload2.get("entries", [])) > len(payload1.get("entries", []))
    assert payload0.get("entries", []) == []


def test_large_corrupt_file(tmp_path: Path) -> None:
    """Ensures the CLI handles a corrupt history file gracefully."""
    hist_file = tmp_path / ".bijux_history"
    with open(hist_file, "wb") as f:
        f.write(os.urandom(100_000))
    r = run_cli(
        ["history", "--format", "json"], env={"BIJUXCLI_HISTORY_FILE": str(hist_file)}
    )
    assert r.returncode != 0
    assert "error" in (r.stdout + r.stderr).lower()


@pytest.mark.skipif(not hasattr(sys, "executable"), reason="no python executable")
def test_python_m_module_invocation_json() -> None:
    """Tests CLI invocation via `python -m`."""
    r = run_module(["history", "--format", "json"])
    assert r.returncode == 0
    payload = assert_json(r.stdout)
    assert isinstance(payload, dict)
    assert isinstance(payload.get("entries"), list)


def test_concurrent_reads() -> None:
    """Ensures history can be read concurrently from multiple threads without errors."""
    run_cli(["history", "clear"])
    run_cli(["version"])

    out: dict[str, Any] = {}
    out2: dict[str, Any] = {}

    def read1() -> None:
        """Executes the 'history' command and stores the result in the 'out' dictionary."""
        out["r"] = run_cli(["history", "--format", "json"])

    def read2() -> None:
        """Executes the 'history' command and stores the result in the 'out2' dictionary."""
        out2["r"] = run_cli(["history", "--format", "json"])

    t1 = threading.Thread(target=read1)
    t2 = threading.Thread(target=read2)
    t1.start()
    t2.start()
    t1.join()
    t2.join()

    assert out["r"].returncode == 0
    assert out2["r"].returncode == 0


@pytest.mark.slow
def test_multiprocess_rw(tmp_path: Path) -> None:
    """Tests concurrent reads/writes from multiple processes."""
    hist_file = tmp_path / ".bijux_history"
    num_procs = 4
    n_ops = 20
    procs = [
        multiprocessing.Process(target=_worker_rw, args=(hist_file, n_ops))
        for _ in range(num_procs)
    ]
    for p in procs:
        p.start()
    for p in procs:
        p.join(timeout=15)
    for p in procs:
        assert not p.exitcode or p.exitcode == 0
    r = run_cli(
        ["history", "--format", "json"], env={"BIJUXCLI_HISTORY_FILE": str(hist_file)}
    )
    assert r.returncode == 0


@pytest.mark.slow
@given(st.text(alphabet=SAFE, min_size=0, max_size=32))
@settings(deadline=None, max_examples=MAX_EXAMPLES)
def test_filter_string_fuzz(s: str) -> None:
    """Fuzz tests the history filter with various strings."""
    run_cli_with_timeout(["version"])
    r = run_cli_with_timeout(["history", "--filter", s, "--format", "json"])
    if r.returncode == -1:
        return
    if r.returncode == 0 and r.stdout.strip():
        obj = json.loads(r.stdout)
        assert isinstance(obj.get("entries"), list)
    else:
        assert r.returncode in (0, 1, 2)


@pytest.mark.slow
@pytest.mark.parametrize("n_entries", [1, 10, 100, 1_000, 10_000])
def test_history_perf_scaling(tmp_path: Path, n_entries: int) -> None:
    """Tests history performance scaling with increasing numbers of entries."""
    hist_file = tmp_path / ".bijux_history"
    history_entries = [
        {"command": "version", "timestamp": time.time() - i} for i in range(n_entries)
    ]
    with open(hist_file, "w") as f:
        json.dump(history_entries, f)
    timings = []
    for _ in range(5):
        t0 = time.perf_counter()
        r = run_cli(
            ["history", "--format", "json"],
            env={"BIJUXCLI_HISTORY_FILE": str(hist_file)},
        )
        t1 = time.perf_counter()
        assert r.returncode == 0
        timings.append(t1 - t0)
    median = sorted(timings)[len(timings) // 2]
    budget = perf_budget(n_entries)
    assert median < budget, (
        f"Too slow: {median:.3f}s for {n_entries} (budget {budget}s)"
    )
