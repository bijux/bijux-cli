# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi
# ruff: noqa: S101, S607

"""End-to-end tests for `bijux repl` command exit."""

from __future__ import annotations

import os
from pathlib import Path
import signal
from subprocess import PIPE, Popen
import time

import pytest

from tests.e2e.conftest import run_cli


@pytest.fixture
def env(tmp_path: Path) -> dict[str, str]:
    """Provides an isolated environment for the REPL tests."""
    return {
        "BIJUXCLI_CONFIG": str(tmp_path / ".env"),
        "BIJUXCLI_TEST_MODE": "1",
    }


@pytest.mark.parametrize("blank_count", range(1, 31))
def test_blank_lines_before_exit(blank_count: int, env: dict[str, str]) -> None:
    """Verify the REPL exits cleanly after multiple blank lines."""
    script = "\n" * blank_count + "exit\n"
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0


@pytest.mark.parametrize("ctrlc_count", range(1, 21))
def test_inline_ctrlc_before_exit(ctrlc_count: int, env: dict[str, str]) -> None:
    """Verify the REPL ignores inline Ctrl-C characters and exits cleanly."""
    script = "\x03" * ctrlc_count + "exit\n"
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0


@pytest.mark.parametrize("ctrld_count", range(1, 11))
def test_inline_ctrld_before_exit(ctrld_count: int, env: dict[str, str]) -> None:
    """Verify the REPL ignores inline Ctrl-D characters and exits cleanly."""
    script = "\x04" * ctrld_count + "exit\n"
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0


ARROW_SEQS = ["\x1b[A", "\x1b[B", "\x1b[C", "\x1b[D"]


@pytest.mark.parametrize("seq", ARROW_SEQS)
@pytest.mark.parametrize("count", range(1, 6))
def test_arrow_key_sequences_before_exit(
    seq: str, count: int, env: dict[str, str]
) -> None:
    """Ensure arrow key sequences don't prevent a clean exit."""
    script = seq * count + "exit\n"
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0


@pytest.mark.parametrize("spaces", range(1, 14))
def test_whitespace_only_before_exit(spaces: int, env: dict[str, str]) -> None:
    """Test that leading whitespace before the exit command is handled."""
    script = " " * spaces + "exit\n"
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0


@pytest.mark.parametrize("cmd", ["exit", "quit"])
def test_immediate_exit_commands(cmd: str, env: dict[str, str]) -> None:
    """Check that both 'exit' and 'quit' commands work correctly."""
    res = run_cli(["repl"], env=env, input_data=f"{cmd}\n")
    assert res.returncode == 0


@pytest.mark.parametrize(
    "sig",
    [signal.SIGINT, signal.SIGTERM, signal.SIGHUP, signal.SIGQUIT, signal.SIGUSR1],
)
def test_repl_handles_posix_signal(sig: signal.Signals, tmp_path: Path) -> None:
    """Verify the REPL exits gracefully when sent a real POSIX signal."""
    env = os.environ.copy()
    env["BIJUXCLI_CONFIG"] = str(tmp_path / ".env")
    env["BIJUXCLI_TEST_MODE"] = "1"

    proc = Popen(  # noqa: S603
        ["bijux", "repl"],
        env=env,
        stdin=PIPE,
        stdout=PIPE,
        stderr=PIPE,
        text=True,
    )

    time.sleep(0.1)

    proc.send_signal(sig)

    out, err = proc.communicate(timeout=2)

    expected = {0, -sig.value, 128 + sig.value}
    if sig is signal.SIGINT:
        expected.add(1)

    assert proc.returncode in expected, (
        f"got {proc.returncode}, expected one of {sorted(expected)}"
    )

    combined = (out or "") + (err or "")
    assert "Exiting REPL." in combined or proc.returncode != 0
