# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra process module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

import sys
import types
from types import SimpleNamespace
from typing import Any
from unittest.mock import MagicMock, call

import pytest

from bijux_cli.core.exceptions import BijuxError
from bijux_cli.infra.process import ProcessPool


class FakeExecutor:
    """A mock ProcessPoolExecutor that records calls."""

    last_max_workers: int | None = None
    last_shutdown_wait: bool | None = None

    def __init__(self, max_workers: int | None = None) -> None:
        """Initialize the fake executor."""
        FakeExecutor.last_max_workers = max_workers
        self.shutdown_called = False

    def shutdown(self, wait: bool = True) -> None:
        """Simulate the shutdown method."""
        self.shutdown_called = True
        FakeExecutor.last_shutdown_wait = wait


class FakeObservability:
    """A mock observability service that records log calls."""

    def __init__(self) -> None:
        """Initialize the fake observer."""
        self.calls: list[tuple[str, str, dict[str, Any] | None]] = []

    def log(self, level: str, msg: str, *, extra: dict[str, Any] | None = None) -> None:
        """Record a log call."""
        self.calls.append((level, msg, extra))


def install_validate(monkeypatch: pytest.MonkeyPatch, func: Any) -> None:
    """Inject a fake validate_command function into a mock module."""
    mod = types.ModuleType("bijux_cli.services.utils")
    mod.validate_command = func  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "bijux_cli.services.utils", mod)


@pytest.fixture
def fake_obs() -> FakeObservability:
    """Provide a FakeObservability instance."""
    return FakeObservability()


@pytest.fixture
def fake_tel() -> MagicMock:
    """Provide a mock telemetry object."""
    return MagicMock()


def test_run_success_and_cache_hit(
    monkeypatch: pytest.MonkeyPatch,
    fake_obs: FakeObservability,
    fake_tel: MagicMock,
) -> None:
    """Test a successful command run and a subsequent cache hit."""
    install_validate(monkeypatch, lambda cmd: cmd)

    run_calls = {"count": 0}

    def fake_run(
        cmd: list[str], capture_output: bool, check: bool, shell: bool
    ) -> SimpleNamespace:
        run_calls["count"] += 1
        return SimpleNamespace(returncode=0, stdout=b"OK", stderr=b"")

    monkeypatch.setattr("bijux_cli.infra.process.subprocess.run", fake_run)

    pool = ProcessPool(fake_obs, fake_tel, max_workers=3)  # type: ignore[arg-type]
    rc, out, err = pool.run(["echo", "x"], executor="unit")

    assert rc == 0
    assert out == b"OK"
    assert err == b""
    fake_tel.event.assert_has_calls(
        [
            call("procpool_execute", {"cmd": ["echo", "x"], "executor": "unit"}),
            call(
                "procpool_executed",
                {"cmd": ["echo", "x"], "executor": "unit", "returncode": 0},
            ),
        ]
    )
    assert ("info", "Process-pool executing", {"cmd": ["echo", "x"]}) in fake_obs.calls
    assert run_calls["count"] == 1

    rc2, out2, err2 = pool.run(["echo", "x"], executor="unit")
    assert (rc2, out2, err2) == (0, b"OK", b"")
    assert run_calls["count"] == 1
    fake_tel.event.assert_any_call(
        "procpool_cache_hit", {"cmd": ["echo", "x"], "executor": "unit"}
    )
    assert ("debug", "Process-pool cache hit", {"cmd": ["echo", "x"]}) in fake_obs.calls

    assert pool.get_status() == {"commands_processed": 1}


def test_run_validation_failure(
    monkeypatch: pytest.MonkeyPatch,
    fake_obs: FakeObservability,
    fake_tel: MagicMock,
) -> None:
    """Test that a command validation failure is handled correctly."""

    def bad_validate(cmd: list[str]) -> None:
        raise BijuxError("invalid")

    install_validate(monkeypatch, bad_validate)

    called = {"run": False}

    def fake_run(*a: Any, **k: Any) -> None:
        called["run"] = True
        raise AssertionError(
            "subprocess.run should not be reached on validation failure"
        )

    monkeypatch.setattr("bijux_cli.infra.process.subprocess.run", fake_run)

    pool = ProcessPool(fake_obs, fake_tel, max_workers=2)  # type: ignore[arg-type]

    with pytest.raises(BijuxError, match="invalid"):
        pool.run(["bad", "cmd"], executor="unit")

    fake_tel.event.assert_any_call(
        "procpool_execution_failed",
        {"cmd": ["bad", "cmd"], "executor": "unit", "error": "validation"},
    )
    assert not called["run"]


def test_run_subprocess_exception_wrapped(
    monkeypatch: pytest.MonkeyPatch,
    fake_obs: FakeObservability,
    fake_tel: MagicMock,
) -> None:
    """Test that an exception from subprocess.run is wrapped in a BijuxError."""
    install_validate(monkeypatch, lambda cmd: cmd)

    def boom(*a: Any, **k: Any) -> None:
        raise ValueError("boom")

    monkeypatch.setattr("bijux_cli.infra.process.subprocess.run", boom)

    pool = ProcessPool(fake_obs, fake_tel)  # type: ignore[arg-type]

    with pytest.raises(BijuxError, match="Process-pool execution failed:"):
        pool.run(["ls"], executor="unit")

    fake_tel.event.assert_any_call(
        "procpool_execution_failed",
        {"cmd": ["ls"], "executor": "unit", "error": "boom"},
    )


def test_lru_eviction_via_max_cache_override(
    monkeypatch: pytest.MonkeyPatch,
    fake_obs: FakeObservability,
    fake_tel: MagicMock,
) -> None:
    """Test that the LRU cache evicts items when its max size is reached."""
    install_validate(monkeypatch, lambda cmd: cmd)

    counter = {"n": 0}

    def fake_run(
        cmd: list[str], capture_output: bool, check: bool, shell: bool
    ) -> SimpleNamespace:
        counter["n"] += 1
        return SimpleNamespace(
            returncode=0, stdout=f"ok{counter['n']}".encode(), stderr=b""
        )

    monkeypatch.setattr("bijux_cli.infra.process.subprocess.run", fake_run)

    pool = ProcessPool(fake_obs, fake_tel)  # type: ignore[arg-type]
    pool._MAX_CACHE = 2  # pyright: ignore[reportAttributeAccessIssue]

    pool.run(["c1"], executor="unit")
    pool.run(["c2"], executor="unit")
    assert pool.get_status() == {"commands_processed": 2}

    pool.run(["c3"], executor="unit")
    assert pool.get_status() == {"commands_processed": 2}
