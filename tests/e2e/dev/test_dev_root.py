# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux dev` command."""

from __future__ import annotations

from pathlib import Path
import sys

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.conftest import run_cli

from .conftest import (
    assert_json,
    assert_no_stacktrace,
    assert_yaml,
    normalize_root,
    run_module,
)

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
]


def pad(s: str) -> str:
    """Pads the given string with two spaces on each side."""
    return f"  {s}  "


@settings(deadline=None, max_examples=10)
@given(st.sampled_from(["json", "yaml"]).map(pad))
def test_format_with_whitespace(fmt: str) -> None:
    """Ensure format flag works with leading/trailing whitespace."""
    r = run_cli(["dev", "--format", fmt])
    assert r.returncode in (0, 2, 3)
    if r.returncode == 0 and "{" in r.stdout:
        obj = assert_json(r.stdout)
        assert obj["status"] == "ok"
    assert_no_stacktrace(r.stdout + r.stderr)


@settings(deadline=None, max_examples=10)
@given(
    st.lists(
        st.sampled_from(
            ["--pretty", "--no-pretty", "-v", "--verbose", "-d", "--debug"]
        ),
        unique=True,
    )
)
def test_flag_permutations(flags: list[str]) -> None:
    """Test various combinations of view-related flags."""
    argv = ["dev", "--format", "json", *flags]
    r = run_cli(argv)
    assert r.returncode in (0, 3)
    assert_no_stacktrace(r.stdout + r.stderr)


def test_help_mentions_all_flags() -> None:
    """Check that the help message contains all required flags."""
    r = run_cli(["dev", "--help"])
    assert r.returncode == 0
    t = r.stdout.lower()
    for flag in REQUIRED_FLAGS:
        assert flag in t
    assert t.startswith("usage:")
    assert len(r.stdout.splitlines()) <= 50


@pytest.mark.parametrize("flag", REQUIRED_FLAGS)
def test_flags_individually(flag: str) -> None:
    """Test each required flag individually for basic correctness."""
    argv = (
        ["dev", "--format", flag] if flag in ("yaml", "JSON", "json") else ["dev", flag]
    )
    r = run_cli(argv)

    if flag in ("-q", "--quiet"):
        assert r.returncode == 0
        assert not r.stdout.strip()

    elif flag in ("-h", "--help"):
        assert r.returncode == 0
        assert r.stdout.lower().startswith("usage:")

    elif flag in ("-f", "--format"):
        assert r.returncode == 2
        obj = assert_json(r.stdout)
        assert obj["code"] == 2

    elif flag == "yaml":
        assert r.returncode == 0
        assert "status:" in r.stdout.lower()

    elif flag in ("json", "JSON"):
        assert r.returncode == 0
        obj = assert_json(r.stdout)
        assert obj["status"] == "ok"

    else:
        assert r.returncode == 0
        out = r.stdout.strip()
        if out.startswith("{"):
            obj = assert_json(out)
            assert obj["status"] == "ok"
        else:
            assert "status:" in out.lower()

    assert_no_stacktrace(r.stdout + r.stderr)


def test_duplicate_format_last_wins_json() -> None:
    """Ensure the last format flag provided takes precedence."""
    r = run_cli(["dev", "--format", "yaml", "--format", "json"])
    obj = assert_json(r.stdout)
    assert obj["status"] == "ok"


def test_quiet_precedence() -> None:
    """Ensure --quiet suppresses output even when a format is specified."""
    r = run_cli(["dev", "--quiet", "--format", "json"])
    assert r.returncode == 0
    assert not r.stdout.strip()


def test_debug_stderr() -> None:
    """Check that --debug prints diagnostic information to stderr."""
    r = run_cli(["dev", "--debug", "--format", "json"])
    assert "Diagnostics: emitted payload" in r.stderr


def test_quiet_debug_stderr() -> None:
    """Ensure --quiet also suppresses the --debug stderr output."""
    r = run_cli(["dev", "--quiet", "--debug", "--format", "json"])
    assert r.returncode == 0
    assert not r.stdout.strip()
    assert not r.stderr.strip()


def test_root_json_golden(golden_dir: Path) -> None:
    """Compare the root JSON output against a golden file."""
    r = run_cli(["dev", "--format", "json"])
    live = normalize_root(assert_json(r.stdout))
    expected = normalize_root(
        assert_json((golden_dir / "root_success.json").read_text())
    )
    assert live == expected


def test_root_yaml_golden(golden_dir: Path) -> None:
    """Compare the root YAML output against a golden file."""
    r = run_cli(["dev", "--format", "yaml"])
    live = normalize_root(assert_yaml(r.stdout))
    expected = normalize_root(
        assert_yaml((golden_dir / "root_success.yaml").read_text())
    )
    assert live == expected


@pytest.mark.skipif(not hasattr(sys, "executable"), reason="No python executable")
def test_python_m_module_invocation_json() -> None:
    """Test invoking the CLI as a module with JSON output."""
    r = run_module(["dev", "--format", "json"])
    assert r.returncode == 0
    obj = assert_json(r.stdout)
    assert obj["status"] == "ok"
    assert_no_stacktrace(r.stdout + r.stderr)


@pytest.mark.skipif(not hasattr(sys, "executable"), reason="No python executable")
def test_python_m_module_invocation_quiet() -> None:
    """Test invoking the CLI as a module with the quiet flag."""
    r = run_module(["dev", "--quiet", "--format", "json"])
    assert r.returncode == 0
    assert not r.stdout.strip()
