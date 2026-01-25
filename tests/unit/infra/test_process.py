# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra process module."""

from __future__ import annotations

import sys
import types
from types import SimpleNamespace
from typing import Any, cast
from unittest.mock import MagicMock, call, patch

import pytest

from bijux_cli.infra.process import ProcessPool, validate_command
from bijux_cli.services.contracts import ObservabilityProtocol, TelemetryProtocol


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
    mod = types.ModuleType("bijux_cli.infra.process")
    mod.validate_command = (  # type: ignore[attr-defined]
        lambda cmd, **kwargs: func(cmd)
    )
    monkeypatch.setitem(sys.modules, "bijux_cli.infra.process", mod)


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

    pool = ProcessPool(
        cast(ObservabilityProtocol, fake_obs),
        cast(TelemetryProtocol, fake_tel),
        max_workers=3,
        allowed_commands=["echo"],
    )
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
        raise ValueError("invalid")

    install_validate(monkeypatch, bad_validate)

    called = {"run": False}

    def fake_run(*a: Any, **k: Any) -> None:
        called["run"] = True
        raise AssertionError(
            "subprocess.run should not be reached on validation failure"
        )

    monkeypatch.setattr("bijux_cli.infra.process.subprocess.run", fake_run)

    pool = ProcessPool(
        cast(ObservabilityProtocol, fake_obs),
        cast(TelemetryProtocol, fake_tel),
        max_workers=2,
        allowed_commands=["echo"],
    )

    with pytest.raises(ValueError, match="invalid"):
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
    """Test that an exception from subprocess.run is wrapped in a RuntimeError."""
    install_validate(monkeypatch, lambda cmd: cmd)

    def boom(*a: Any, **k: Any) -> None:
        raise ValueError("boom")

    monkeypatch.setattr("bijux_cli.infra.process.subprocess.run", boom)

    pool = ProcessPool(
        cast(ObservabilityProtocol, fake_obs),
        cast(TelemetryProtocol, fake_tel),
        max_workers=2,
        allowed_commands=["ls"],
    )

    with pytest.raises(RuntimeError, match="Process-pool execution failed:"):
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

    pool = ProcessPool(
        cast(ObservabilityProtocol, fake_obs),
        cast(TelemetryProtocol, fake_tel),
        max_workers=2,
        allowed_commands=["c1", "c2", "c3"],
    )
    pool._MAX_CACHE = 2

    pool.run(["c1"], executor="unit")
    pool.run(["c2"], executor="unit")
    assert pool.get_status() == {"commands_processed": 2}

    pool.run(["c3"], executor="unit")
    assert pool.get_status() == {"commands_processed": 2}


def test_validate_command_empty() -> None:
    """Test that providing an empty command list raises an error."""
    with pytest.raises(ValueError, match="invalid command"):
        validate_command([], allowed_commands=["echo"])


def test_validate_command_not_allowed() -> None:
    """Test that a command not in the allowed list is rejected."""
    with pytest.raises(ValueError, match="not in allowed list"):
        validate_command(["cat", "file.txt"], allowed_commands=["echo", "ls"])


@patch("shutil.which")
def test_validate_command_not_found(
    mock_which: MagicMock,
) -> None:
    """Test that a command not found on the system PATH is rejected."""
    mock_which.return_value = None
    with pytest.raises(ValueError, match="not found|not executable"):
        validate_command(["cat", "file.txt"], allowed_commands=["cat"])


@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_disallowed_path(
    mock_basename: MagicMock, mock_which: MagicMock
) -> None:
    """Test that a command whose resolved path does not match the command name is rejected."""
    mock_which.return_value = "/bin/cat2"
    mock_basename.side_effect = lambda x: "cat" if x == "cat" else "cat2"
    with pytest.raises(ValueError, match="Disallowed command path"):
        validate_command(["cat", "file.txt"], allowed_commands=["cat"])


@pytest.mark.parametrize("unsafe_char", [";", "|", "&", ">", "<", "`", "!"])
@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_unsafe_arg(
    mock_basename: MagicMock,
    mock_which: MagicMock,
    unsafe_char: str,
) -> None:
    """Test that command arguments containing unsafe characters are rejected."""
    mock_which.return_value = "/bin/echo"
    mock_basename.side_effect = lambda x: "echo"
    with pytest.raises(ValueError, match="Unsafe argument"):
        validate_command(["echo", f"test{unsafe_char}"], allowed_commands=["echo"])


@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_success(
    mock_basename: MagicMock, mock_which: MagicMock
) -> None:
    """Test that a valid and safe command is successfully validated and resolved."""
    mock_which.return_value = "/bin/echo"
    mock_basename.side_effect = lambda x: "echo"
    cmd = ["echo", "hello"]
    result = validate_command(cmd, allowed_commands=["echo"])
    assert result == ["/bin/echo", "hello"]


@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_success_full_path(
    mock_basename: MagicMock, mock_which: MagicMock
) -> None:
    """Test that a command provided with a full path is validated correctly."""
    mock_which.return_value = "/bin/echo"
    mock_basename.side_effect = lambda x: "echo"
    cmd = ["/bin/echo", "hello"]
    result = validate_command(cmd, allowed_commands=["echo"])
    assert result == ["/bin/echo", "hello"]


def test_validate_command_custom_env() -> None:
    """Test that a disallowed command is rejected."""
    with pytest.raises(ValueError, match="not in allowed list"):
        validate_command(["echo", "test"], allowed_commands=["custom_cmd"])


@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_default_env(
    mock_basename: MagicMock, mock_which: MagicMock
) -> None:
    """Test that the allowlist is enforced for valid commands."""
    mock_which.return_value = "/bin/grep"
    mock_basename.side_effect = lambda x: "grep"
    result = validate_command(["grep", "pattern"], allowed_commands=["grep"])
    assert result == ["/bin/grep", "pattern"]
