# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux dev` command."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
from typing import Any

import pytest
import yaml


def run_module(
    argv: Sequence[str], env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    """Invoke `python -m bijux_cli ...` to simulate an installed CLI module entrypoint.

    Args:
        argv: A sequence of command-line arguments to pass to the CLI.
        env: An optional dictionary of environment variables.

    Returns:
        The `subprocess.CompletedProcess` instance resulting from the command execution.
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
    """Skip the current test if the platform does not support symlinks.

    Args:
        tmp_path: The pytest `tmp_path` fixture for creating temporary files.
    """
    try:
        (tmp_path / "probe").symlink_to(tmp_path / "target")
    except OSError:
        pytest.skip("symlinks not supported on this platform")


@pytest.fixture(scope="session")
def golden_dir() -> Path:
    """Provide the path to the golden test_fixtures' directory.

    Returns:
        A `pathlib.Path` object pointing to the golden test_fixtures directory.
    """
    return Path(__file__).parent.parent / "test_fixtures" / "dev"


@pytest.fixture(autouse=True)
def clean_env(monkeypatch: pytest.MonkeyPatch) -> None:
    """Keep the environment deterministic across tests.

    This fixture removes potentially interfering `BIJUXCLI_*` environment
    variables before each test to ensure isolation.

    Args:
        monkeypatch: The pytest `monkeypatch` fixture.
    """
    monkeypatch.delenv("BIJUXCLI_CONFIG", raising=False)
    monkeypatch.delenv("BIJUXCLI_DI_LIMIT", raising=False)
    monkeypatch.delenv("BIJUXCLI_DEV_MODE", raising=False)
    monkeypatch.setenv("LC_ALL", "C")
    monkeypatch.setenv("LANG", "C")


def assert_json(text: str) -> Any:
    """Assert that the input text is valid JSON and return the parsed object.

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
    """Assert that the input text is valid YAML and return the parsed object.

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
    s = text.lower()
    assert "traceback" not in s
    assert "typer" not in s
    assert "click" not in s


def assert_error_contract(obj: Mapping[str, Any], code: int | None = None) -> None:
    """Assert that an object conforms to the standard error shape.

    Args:
        obj: The parsed JSON/YAML object to check.
        code: The optional expected integer error code.
    """
    assert isinstance(obj, dict)
    assert "error" in obj
    if code is not None:
        assert obj.get("code") == code


def normalize_root(payload: Mapping[str, Any]) -> Mapping[str, Any]:
    """Return a stable snapshot of `bijux dev` output for comparison.

    This removes environment-specific fields to allow for deterministic testing.

    Args:
        payload: The parsed JSON/YAML output from the `bijux dev` command.

    Returns:
        A normalized dictionary containing only the stable fields.
    """
    out: dict[str, Any] = {"status": payload.get("status")}
    if "mode" in payload:
        out["mode"] = payload["mode"]
    return out


def normalize_di(payload: Mapping[str, Any]) -> Mapping[str, Any]:
    """Return a stable snapshot of a DI graph, focusing on shape not identity.

    This function anonymizes the specific implementation details in a DI graph
    to allow for stable, structural comparisons in tests.

    Args:
        payload: The parsed JSON/YAML output from the `bijux dev di` command.

    Returns:
        A normalized dictionary representing the structure of the DI graph.
    """

    def norm_list(
        lst: list[dict[str, Any]] | None,
    ) -> list[dict[str, str | None]]:
        """Normalize a list of DI entries to a stable representation."""
        out: list[dict[str, str | None]] = []
        for item in lst or []:
            if "implementation" in item:
                out.append(
                    {
                        "protocol": "<str>" if "protocol" in item else None,
                        "alias": "<str|null>" if "alias" in item else None,
                        "implementation": "<null>",
                    }
                )
            else:
                out.append(
                    {
                        "protocol": "<str>" if "protocol" in item else None,
                        "alias": "<str|null>" if "alias" in item else None,
                    }
                )
        return sorted(out, key=lambda d: tuple(sorted(d.keys())))

    return {
        "factories": norm_list(payload.get("factories", [])),
        "services": norm_list(payload.get("services", [])),
    }
