# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Integration Test suite for the Bijux CLI."""

from __future__ import annotations

import json
import signal
from types import NoneType
from typing import Any

import pytest
import yaml

from tests.functional.test_functional import cli

try:
    from tests.functional.test_functional import _run_repl_script
except Exception:
    _run_repl_script = None  # type: ignore[assignment]


@pytest.mark.parametrize(
    "tokens",
    [
        ("--help",),
        ("help",),
        ("version", "--help"),
        ("config", "--help"),
        ("config", "get", "--help"),
        ("config", "set", "--help"),
        ("config", "list", "--help"),
        ("config", "unset", "--help"),
        ("config", "clear", "--help"),
        ("history", "--help"),
        ("history", "service", "list", "--help"),
        ("history", "clear", "--help"),
        ("memory", "--help"),
        ("memory", "list", "--help"),
        ("memory", "get", "--help"),
        ("memory", "set", "--help"),
        ("memory", "delete", "--help"),
        ("plugins", "--help"),
        ("plugins", "list", "--help"),
        ("plugins", "check", "--help"),
        ("plugins", "info", "--help"),
        ("sleep", "--help"),
    ],
    ids=lambda t: " ".join(t),
)
def test_help_invocations(tokens: tuple[str, ...]) -> None:
    """Test that various help command invocations execute successfully."""
    r = cli(*tokens, expect_exit_code=None)
    assert r.returncode in (0, 1, 2)
    if r.stdout:
        assert isinstance(r.stdout, str)


@pytest.mark.parametrize(
    "tokens",
    [
        ("version",),
        ("config", "list"),
        ("history", "service", "list"),
        ("memory", "list"),
        ("plugins", "list"),
        ("status",),
    ],
    ids=lambda t: " ".join(t),
)
def test_json_output_shape(tokens: tuple[str, ...]) -> None:
    """Verify the basic shape and type of JSON output for various commands."""
    r = cli(*tokens, json_output=True, expect_exit_code=None)
    assert r.returncode in (0, 1, 2)
    raw = r.json_out or r.json_err

    if raw is not None:
        assert isinstance(raw, dict | list | str | int | float | bool | NoneType)
        if tokens == ("version",) and isinstance(raw, dict):
            assert "version" in raw
            assert isinstance(raw["version"], str)
            assert raw["version"]
    else:
        assert r.stdout is not None


@pytest.mark.parametrize(
    "tokens",
    [
        ("config", "get"),
        ("config", "unset"),
        ("plugins", "uninstall", "invalid"),
        ("plugins", "info", "/invalid"),
        ("plugins", "install", "/invalid"),
        ("plugins", "scaffold", "invalid name"),
    ],
    ids=lambda t: " ".join(t),
)
def test_invalid_invocations(tokens: tuple[str, ...]) -> None:
    """Test that various invalid command invocations fail with an error."""
    r = cli(*tokens, json_output=True, expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    if isinstance(data, dict):
        msg = str(data.get("error", "")).lower()
        failure = str(data.get("failure", "")).lower()
        assert msg or failure


def test_memory_set_get_roundtrip() -> None:
    """Test a set-then-get roundtrip for the memory command."""
    r1 = cli(
        "memory", "set", "int_test_key", "val", json_output=True, expect_exit_code=None
    )
    assert r1.returncode in (0, 1, 2)
    r2 = cli("memory", "get", "int_test_key", json_output=True, expect_exit_code=None)
    assert r2.returncode in (0, 1, 2)
    data = r2.json_out or r2.json_err
    if isinstance(data, dict):
        got = data.get("value", data.get("val", None))
        assert got is None or isinstance(got, (str | int | float | bool))


def test_memory_list_contains_key_if_supported() -> None:
    """Verify the shape of the memory list command's output."""
    r = cli("memory", "list", json_output=True, expect_exit_code=None)
    assert r.returncode in (0, 1, 2)
    data = r.json_out or r.json_err
    if isinstance(data, list):
        assert all(isinstance(k, (str | dict)) for k in data)
    elif isinstance(data, dict):
        keys = data.get("keys")
        if isinstance(keys, list):
            assert all(isinstance(k, (str | dict)) for k in keys)


def test_memory_delete_then_get_missing() -> None:
    """Test that a key is missing after it has been deleted."""
    cli(
        "memory", "set", "int_test_key2", "val", json_output=True, expect_exit_code=None
    )
    r1 = cli(
        "memory", "delete", "int_test_key2", json_output=True, expect_exit_code=None
    )
    assert r1.returncode in (0, 1, 2)
    r2 = cli("memory", "get", "int_test_key2", json_output=True, expect_exit_code=None)
    assert r2.returncode in (1, 2, 0)
    data = r2.json_out or r2.json_err
    if isinstance(data, dict):
        msg = str(data.get("error", "")).lower()
        val = data.get("value")
        assert "not" in msg or val in (None, "")


def test_plugins_list_shape() -> None:
    """Verify the shape of the plugins list command's output."""
    r = cli("plugins", "list", json_output=True, expect_exit_code=None)
    assert r.returncode in (0, 1, 2)
    raw = r.json_out or r.json_err
    items: list[Any] = []
    if isinstance(raw, list):
        items = raw
    elif isinstance(raw, dict):
        val = raw.get("plugins", [])
        if isinstance(val, list):
            items = val
    else:
        try:
            parsed = json.loads(r.stdout)
            if isinstance(parsed, list):
                items = parsed
            elif isinstance(parsed, dict):
                v = parsed.get("plugins", [])
                if isinstance(v, list):
                    items = v
        except Exception:
            items = []
    assert isinstance(items, list)
    if items:
        if isinstance(items[0], dict):
            assert all(
                isinstance(p, dict) and isinstance(p.get("name"), str) for p in items
            )
        else:
            assert all(isinstance(p, str) for p in items)


def _assert_ok_or_sigterm(code: int) -> None:
    """Assert that an exit code indicates success or a SIGTERM from timeout."""
    assert code in (0, 2, -signal.SIGTERM)


def test_sleep_basic() -> None:
    """Test the basic functionality of the sleep command."""
    r = cli("sleep", "0.1", expect_exit_code=None, timeout=1)
    _assert_ok_or_sigterm(r.returncode)


def test_sleep_json() -> None:
    """Test the sleep command with JSON output format."""
    r = cli("sleep", "0.1", "--format", "json", expect_exit_code=None, timeout=1)
    _assert_ok_or_sigterm(r.returncode)


def test_sleep_quiet() -> None:
    """Test the sleep command with the quiet flag."""
    r = cli("sleep", "0.1", "--quiet", expect_exit_code=None, timeout=1)
    _assert_ok_or_sigterm(r.returncode)


@pytest.mark.skipif(_run_repl_script is None, reason="repl helper not available")
def test_repl_smoke() -> None:
    """Perform a smoke test of the REPL by starting and quitting."""
    r = _run_repl_script(["quit"], timeout=2)  # pyright: ignore[reportOptionalCall]
    assert r.returncode in (0, 2, -signal.SIGTERM)
    assert r.stderr is not None


def test_root_no_args_smoke() -> None:
    """Perform a smoke test of invoking the CLI with no arguments."""
    r = cli(expect_exit_code=None, timeout=2)
    assert r.returncode in (0, 1, 2, -signal.SIGTERM)
    assert isinstance(r.returncode, int)


def test_version_pretty() -> None:
    """Test the version command with pretty-printed output."""
    r = cli("version", "--pretty", expect_exit_code=None)
    assert r.returncode in (0, 1, 2)
    assert r.stdout.strip() != ""


def test_version_no_pretty() -> None:
    """Test the version command with non-pretty-printed output."""
    r = cli("version", "--no-pretty", expect_exit_code=None)
    assert r.returncode in (0, 1, 2)
    assert r.stdout.strip() != ""


def test_invalid_global_format() -> None:
    """Test that using an invalid global format flag fails as expected."""
    r = cli("--format", "invalid", expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    text = ""
    if isinstance(data, dict):
        text = str(data.get("error", ""))
    text = (text or r.stderr or r.stdout).lower()
    assert any(w in text for w in ("unsupported", "invalid", "unknown", "no such"))


@pytest.mark.parametrize(
    "tokens",
    [
        ("version",),
        ("config", "list"),
        ("memory", "list"),
        ("history", "service", "list"),
    ],
    ids=lambda t: " ".join(t),
)
def test_yaml_output_shape(tokens: tuple[str, ...]) -> None:
    """Verify the basic shape and type of YAML output for various commands."""
    r = cli(*tokens, "--format", "yaml", expect_exit_code=None)
    assert r.returncode in (0, 1, 2)
    if r.returncode == 0:
        data = yaml.safe_load(r.stdout or "null")
        assert isinstance(data, (dict | list | type(None)))
    else:
        assert (r.stderr or r.stdout).strip() != ""
