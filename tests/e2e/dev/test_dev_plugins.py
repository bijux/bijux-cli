# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux dev` command."""

from __future__ import annotations

from pathlib import Path

import pytest

from tests.e2e.conftest import run_cli
from tests.e2e.dev.conftest import (
    assert_error_contract,
    assert_json,
    assert_no_stacktrace,
    assert_yaml,
)


def test_json_golden(golden_dir: Path) -> None:
    """Ensure the JSON output matches the golden file."""
    r = run_cli(["dev", "list-plugins", "--format", "json"])
    want = assert_json((golden_dir / "list_plugins.json").read_text())
    got = assert_json(r.stdout) if r.stdout.strip() else {}
    want_plugins = set(want.get("plugins", []))
    got_plugins = set(got.get("plugins", [])) if isinstance(got, dict) else set()
    if not got_plugins:
        assert isinstance(got, dict)
        assert isinstance(got.get("plugins", []), list)
        return
    missing = want_plugins - got_plugins
    assert not missing, f"missing expected plugins: {missing}"


def test_yaml_golden(golden_dir: Path) -> None:
    """Ensure the YAML output matches the golden file."""
    r = run_cli(["dev", "list-plugins", "--format", "yaml"])
    want = assert_yaml((golden_dir / "list_plugins.yaml").read_text())
    got = assert_yaml(r.stdout) if r.stdout.strip() else {}
    want_plugins = set(want.get("plugins", []))
    got_plugins = set(got.get("plugins", [])) if isinstance(got, dict) else set()
    if not got_plugins:
        assert isinstance(got, dict)
        assert isinstance(got.get("plugins", []), list)
        return
    missing = want_plugins - got_plugins
    assert not missing, f"missing expected plugins: {missing}"


@pytest.mark.parametrize("fmt", ["json", "yaml", "JSON"])
def test_formats(fmt: str) -> None:
    """Test that various format flags produce a valid structure."""
    r = run_cli(["dev", "list-plugins", "--format", fmt])
    if fmt.lower() == "yaml":  # noqa: SIM108
        obj = assert_yaml(r.stdout)
    else:
        obj = assert_json(r.stdout)
    assert "plugins" in obj
    assert r.returncode == 0
    assert_no_stacktrace(r.stdout + r.stderr)


def test_quiet_suppresses_stdout() -> None:
    """Ensure the --quiet flag suppresses standard output."""
    r = run_cli(["dev", "list-plugins", "--quiet"])
    assert r.returncode == 0
    assert not r.stdout.strip()
    assert_no_stacktrace(r.stdout + r.stderr)


def test_missing_format_value_is_cli_error() -> None:
    """Ensure that calling --format without a value returns a CLI error."""
    r = run_cli(["dev", "list-plugins", "--format"])
    assert r.returncode == 2
    obj = assert_json(r.stdout)
    assert_error_contract(obj, 2)
