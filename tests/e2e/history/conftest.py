# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Helper functions and test_fixtures for end-to-end `bijux history` tests."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from contextlib import suppress
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
from typing import Any

import pytest
import yaml

from tests.e2e.conftest import BIN  # pyright: ignore[reportMissingImports]

SAFE = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_ :/."
MAX_EXAMPLES = int(os.environ.get("BIJUX_TEST_FUZZ_EXAMPLES", "35"))
PER_EX_TIMEOUT = float(os.environ.get("BIJUX_TEST_FUZZ_TIMEOUT", "0.25"))
REQUIRED_ENTRY_KEYS = {"command", "timestamp"}
REQUIRED_FLAGS = [
    "-h",
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
    "--limit",
    "--export",
    "--import",
    "--group-by",
    "--filter",
    "--sort",
]


class Proc:
    """A simple container for holding process execution results.

    Attributes:
        returncode: The integer exit code of the process.
        stdout: The captured standard output as a string.
        stderr: The captured standard error as a string.
    """

    def __init__(self, returncode: int, stdout: str, stderr: str) -> None:
        """Initializes a Proc instance.

        Args:
            returncode: The process exit code.
            stdout: The captured standard output.
            stderr: The captured standard error.
        """
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr


@pytest.fixture(scope="session")
def golden_dir() -> Path:
    """Provide the path to the golden files' directory for snapshot testing.

    Returns:
        A `pathlib.Path` object pointing to the golden test_fixtures directory.
    """
    return Path(__file__).parents[1] / "test_fixtures" / "history"


@pytest.fixture(autouse=True)
def _clean_env(  # pyright: ignore[reportUnusedFunction]
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Set up a clean, isolated environment for each test.

    This auto-use fixture ensures a deterministic environment by setting a consistent
    locale and isolating config/history files to a temporary directory.

    Args:
        monkeypatch: The pytest `monkeypatch` fixture.
        tmp_path: The pytest `tmp_path` fixture.
    """
    monkeypatch.setenv("LC_ALL", "C")
    monkeypatch.setenv("LANG", "C")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(tmp_path / ".env"))
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(tmp_path / ".bijux_history"))
    monkeypatch.delenv("BIJUXCLI_TEST_DISK_FULL", raising=False)
    monkeypatch.setenv("BIJUX_TEST_FUZZ_TIMEOUT", "1.0")


def assert_json(text: str) -> Any:
    """Assert that text is valid JSON and return the parsed object.

    Args:
        text: The string to parse as JSON.

    Returns:
        The parsed Python object from the JSON text.
    """
    try:
        return json.loads(text)
    except Exception as e:
        pytest.fail(f"Not valid JSON: {e}\n{text}")


def assert_yaml(text: str) -> Any:
    """Assert that text is valid YAML and return the parsed object.

    Args:
        text: The string to parse as YAML.

    Returns:
        The parsed Python object from the YAML text.
    """
    try:
        return yaml.safe_load(text) or {}
    except Exception as e:
        pytest.fail(f"Not valid YAML: {e}\n{text}")


def assert_no_stacktrace(text: str) -> None:
    """Assert that no Python traceback or framework names are in the output.

    Args:
        text: The captured stdout/stderr from a CLI command.
    """
    s = (text or "").lower()
    assert "traceback" not in s
    assert "click" not in s
    assert "typer" not in s


def normalize_history_payload(obj: Any) -> list[dict[str, Any]]:
    """Create a deterministic projection of a history object for comparison.

    This helper handles wrapped dictionary structures, ensures the result is a
    list of dictionaries, removes the non-deterministic `timestamp` field, and
    sorts the result for stable equality checks.

    Args:
        obj: The raw parsed JSON or YAML object from the CLI output.

    Returns:
        A normalized and sorted list of history entry dictionaries.
    """
    if isinstance(obj, dict) and "entries" in obj:
        obj = obj["entries"]

    if not isinstance(obj, list):
        return []
    proj: list[dict[str, Any]] = []
    for e in obj:
        if not isinstance(e, Mapping):
            continue
        known_keys = {
            "command",
            "timestamp",
            "success",
            "return_code",
            "duration_ms",
        }
        proj.append({k: e.get(k) for k in known_keys if k in e})
    return sorted(
        proj, key=lambda d: (int(d.get("timestamp", 0)), str(d.get("command", "")))
    )


def ensure_entry_shape(entry: Mapping[str, Any]) -> None:
    """Ensure a history entry has the required keys and correct types.

    Args:
        entry: A dictionary representing a single history entry.
    """
    for k in REQUIRED_ENTRY_KEYS:
        assert k in entry
    assert isinstance(entry["command"], str)
    assert isinstance(entry["timestamp"], (int | float))


def _fuzz_env() -> dict[str, str]:
    """Create a dedicated, isolated environment for fuzz testing."""
    env = os.environ.copy()
    tmpdir = env.get("PYTEST_TMPDIR", tempfile.gettempdir())
    pid = os.getpid()
    hist = Path(tmpdir) / f".bijux_history.fuzz.{pid}.json"

    env.setdefault("BIJUXCLI_CONFIG", str(Path(tmpdir) / f".env.fuzz.{pid}"))
    env.setdefault("BIJUXCLI_HISTORY_FILE", str(hist))
    env["LC_ALL"] = "C"
    env["LANG"] = "C"
    env.setdefault("BIJUX_TEST_FAST", "1")
    return env


def run_cli_with_timeout(args: Sequence[str], timeout: float = PER_EX_TIMEOUT) -> Proc:
    """Invoke the CLI with a timeout, forcefully terminating if exceeded.

    Args:
        args: A sequence of CLI arguments (excluding the binary name).
        timeout: Maximum execution duration in seconds.

    Returns:
        A `Proc` instance containing the result. The return code will be -1
        if a timeout occurred.
    """
    env = _fuzz_env()
    p = subprocess.Popen(  # noqa: S603
        [str(BIN), *args],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )

    stdout, stderr = "", ""
    try:
        stdout, stderr = p.communicate(timeout=timeout)
        rc = p.returncode
    except subprocess.TimeoutExpired:
        with suppress(Exception):
            p.kill()
        with suppress(Exception):
            stdout, stderr = p.communicate(timeout=0.5)
        rc = -1

    return Proc(rc, stdout or "", stderr or "")


def run_module(
    argv: Sequence[str], env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    """Run the CLI as a Python module via `python -m bijux_cli`.

    Args:
        argv: A sequence of command-line arguments to pass to the module.
        env: An optional dictionary of environment variables.

    Returns:
        The `subprocess.CompletedProcess` instance from the execution.
    """
    cmd = [sys.executable, "-m", "bijux_cli", *argv]
    return subprocess.run(  # noqa: S603
        cmd,
        env=env or os.environ.copy(),
        text=True,
        capture_output=True,
        check=False,
    )


def make_ro_dir(path: Path) -> None:
    """Create a read-only directory.

    Args:
        path: The path where the read-only directory should be created.
    """
    path.mkdir(parents=True, exist_ok=True)
    path.chmod(stat.S_IREAD | stat.S_IEXEC)


def require_symlink(tmp_path: Path) -> None:
    """Skip a test if the current platform or filesystem does not support symlinks.

    Args:
        tmp_path: The pytest `tmp_path` fixture for creating temporary files.
    """
    probe = tmp_path / "p"
    link = tmp_path / "l"
    try:
        probe.write_text("x")
        link.symlink_to(probe)
    except OSError as exc:
        pytest.skip(f"symlinks not supported: {exc}")
