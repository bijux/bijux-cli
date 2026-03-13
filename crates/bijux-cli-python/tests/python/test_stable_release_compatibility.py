from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import venv
from pathlib import Path

import pytest

from bijux_cli_py import execution_facade

STABLE_PYPI_VERSION = "0.2.0"
ENABLE_STABLE_RELEASE_CHECK = "BIJUX_ENABLE_STABLE_PYPI_PARITY"


def _project_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _stable_python() -> str:
    temp_root = Path(tempfile.mkdtemp(prefix="bijux-stable-release-"))
    venv_dir = temp_root / "venv"
    venv.EnvBuilder(with_pip=True, clear=True).create(venv_dir)
    python = venv_dir / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    subprocess.run(
        [str(python), "-m", "pip", "install", f"bijux-cli=={STABLE_PYPI_VERSION}"],
        check=True,
        capture_output=True,
        text=True,
    )
    return str(python)


def _run_stable(*args: str) -> subprocess.CompletedProcess[str]:
    python = _stable_python()
    return subprocess.run(
        [python, "-m", "bijux_cli_py", *args],
        check=False,
        capture_output=True,
        text=True,
    )


@pytest.mark.nightly
@pytest.mark.skipif(
    os.environ.get(ENABLE_STABLE_RELEASE_CHECK) != "1",
    reason="set BIJUX_ENABLE_STABLE_PYPI_PARITY=1 to run the PyPI stable compatibility check",
)
def test_current_python_package_keeps_stable_release_command_overlap() -> None:
    stable_help = _run_stable("--help")
    current_help = execution_facade(["--help"])

    assert stable_help.returncode == 0
    assert "Usage:" in stable_help.stdout
    assert "Usage:" in current_help
    for command in ("status", "doctor", "version", "plugins", "config"):
        assert command in stable_help.stdout
        assert command in current_help


@pytest.mark.nightly
@pytest.mark.skipif(
    os.environ.get(ENABLE_STABLE_RELEASE_CHECK) != "1",
    reason="set BIJUX_ENABLE_STABLE_PYPI_PARITY=1 to run the PyPI stable compatibility check",
)
def test_current_python_package_keeps_stable_release_entrypoint_shape() -> None:
    stable_version = _run_stable("version")
    current_version = execution_facade(["version"]).strip()

    assert stable_version.returncode == 0
    assert stable_version.stdout.strip()
    assert current_version
    assert STABLE_PYPI_VERSION in stable_version.stdout
    assert current_version != stable_version.stdout.strip()
