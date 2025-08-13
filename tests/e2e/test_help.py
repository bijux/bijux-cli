# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux help` command."""

from __future__ import annotations

import json
import re
from typing import Any, cast

from hypothesis import HealthCheck, assume, given, settings
from hypothesis import strategies as st
import pytest
import yaml

from tests.e2e.conftest import run_cli

KNOWN_FLAGS = ["--help", "--format", "--pretty", "--no-pretty", "--quiet", "-v", "-d"]


def _normalize(text: str) -> str:
    """Normalize whitespace and case for consistent comparison."""
    return re.sub(r"\s+", " ", text.strip().lower())


def assert_json_obj(text: str) -> dict[str, Any]:
    """Parse JSON and assert the result is a dictionary."""
    obj = json.loads(text)
    assert isinstance(obj, dict), f"Payload is not a dict: {obj!r}"
    return obj


def assert_yaml_obj(text: str) -> dict[str, Any]:
    """Parse YAML and assert the result is a dictionary."""
    obj = yaml.safe_load(text)
    assert isinstance(obj, dict), f"Payload is not a dict: {obj!r}"
    return obj


def assert_no_framework_leak(text: str) -> None:
    """Ensure output contains no stacktraces, Typer, or Click names."""
    lowered = text.lower()
    assert "traceback" not in lowered
    assert "typer" not in lowered
    assert "click" not in lowered


@pytest.mark.parametrize(
    "cmd", ["help", "plugins", "config", "version", "status", "sleep"]
)
def test_help_flag_shows_usage(cmd: str) -> None:
    """Test that `COMMAND --help` shows usage info."""
    res = run_cli([cmd, "--help"])
    assert res.returncode == 0
    assert "usage:" in res.stdout.lower()


def test_help_default_is_human() -> None:
    """Test that `bijux help` defaults to human-readable output."""
    res = run_cli(["help"])
    assert res.returncode == 0
    out = res.stdout.strip()
    assert not out.startswith("{")
    assert not out.startswith("---")
    assert "usage:" in out.lower()


def test_root_help_is_human() -> None:
    """Test that `bijux --help` defaults to human-readable output."""
    res = run_cli(["--help"])
    assert res.returncode == 0
    out = res.stdout.strip()
    assert not out.startswith("{")
    assert not out.startswith("---")
    assert "usage:" in out.lower()


@pytest.mark.parametrize("cmd", ["version", "status", "sleep", "doctor", "help"])
def test_help_command_equivalence(cmd: str) -> None:
    """Test `COMMAND --help` is the same as `help COMMAND`."""
    out1 = run_cli([cmd, "--help"]).stdout
    out2 = run_cli(["help", cmd]).stdout
    assert _normalize(out1) == _normalize(out2)


def test_root_help_equivalence() -> None:
    """Test `bijux --help` is the same as `bijux help`."""
    out1 = run_cli(["--help"]).stdout
    out2 = run_cli(["help"]).stdout
    assert _normalize(out1) == _normalize(out2)


@pytest.mark.parametrize("fmt", ["json", "yaml"])
def test_help_format_json_yaml(fmt: str) -> None:
    """Test that `help --format` works for JSON and YAML."""
    res = run_cli(["help", "--format", fmt])
    assert res.returncode == 0
    obj = assert_json_obj(res.stdout) if fmt == "json" else assert_yaml_obj(res.stdout)
    assert "help" in obj


def test_help_invalid_format() -> None:
    """Test that `help --format bogus` fails gracefully."""
    res = run_cli(["help", "--format", "bogus"])
    assert res.returncode == 2
    err_obj = assert_json_obj(res.stderr)
    assert "error" in err_obj
    assert "format" in err_obj["error"].lower()
    assert err_obj["code"] == 2


def test_help_pretty_and_nopretty_flags() -> None:
    """Test the --pretty and --no-pretty flags."""
    res = run_cli(["help", "--format", "json", "--pretty"])
    assert "\n" in res.stdout
    res2 = run_cli(["help", "--format", "json", "--no-pretty"])
    assert len(res2.stdout.strip().splitlines()) == 1


@pytest.mark.parametrize("flag", ["-v", "-d"])
def test_help_verbose_and_debug_flags(flag: str) -> None:
    """Test that verbose and debug flags add extra context."""
    res = run_cli(["help", "--format", "json", flag])
    obj = assert_json_obj(res.stdout)
    assert "help" in obj
    assert "platform" in obj


def test_help_invalid_command() -> None:
    """Test that `help notacommand` fails gracefully."""
    res = run_cli(["help", "notacommand"])
    assert res.returncode == 2
    err = (res.stdout + res.stderr).lower()
    assert "no such command" in err
    assert_no_framework_leak(res.stdout + res.stderr)


def test_help_nonascii_env(monkeypatch: Any) -> None:
    """Test that non-ASCII environment variables are handled."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/tmp/\u2603")  # noqa: S108
    res = run_cli(["help"])
    assert res.returncode == 3
    err_obj = assert_json_obj(res.stderr)
    assert "ascii" in err_obj["error"].lower()
    assert err_obj["code"] == 3
    assert_no_framework_leak(res.stdout + res.stderr)


@given(
    name=st.text(min_size=1, max_size=8).filter(
        lambda s: "\x00" not in s and any(ord(c) > 127 for c in s)
    )
)
@settings(
    max_examples=10, deadline=None, suppress_health_check=[HealthCheck.filter_too_much]
)
def test_help_unicode_garbage_command(name: str) -> None:
    """Test that non-ASCII command names are handled."""
    assume(any(ord(c) >= 128 for c in name))
    res = run_cli(["help", name])
    out = (res.stdout + res.stderr).lower()
    assert (
        "no such command" in out
        or "no such option" in out
        or "ascii" in out
        or "encoding" in out
    )


@pytest.mark.parametrize(
    "error_flags",
    [
        ["--format", "bogus"],
        ["--unknown-flag"],
        ["--format"],
    ],
)
def test_adr_help_wins_over_errors(error_flags: list[str]) -> None:
    """ADR: --help must short-circuit and ignore all other flags, even invalid ones."""
    argv = ["help", "--help", *error_flags]
    res = run_cli(argv)
    assert res.returncode == 0
    assert "usage:" in res.stdout.lower()
    assert "error" not in res.stdout.lower()
    assert_no_framework_leak(res.stdout + res.stderr)


@pytest.mark.parametrize(
    "output_flags", [["--debug"], ["--verbose"], ["--format", "json"]]
)
def test_adr_quiet_wins_over_output_flags(output_flags: list[str]) -> None:
    """ADR: --quiet must suppress all output from other flags."""
    argv = ["help", "--quiet", *output_flags]
    res = run_cli(argv)
    assert res.returncode == 0
    assert res.stdout == ""
    assert res.stderr == ""


def test_adr_quiet_preserves_error_code_with_no_output(monkeypatch: Any) -> None:
    """ADR: --quiet must suppress error output but preserve the exit code."""
    res_fmt = run_cli(["help", "--quiet", "--format", "bogus"])
    assert res_fmt.returncode == 2
    assert res_fmt.stdout == ""
    assert res_fmt.stderr == ""

    monkeypatch.setenv("BIJUXCLI_BOGUS", "\u2744")
    res_env = run_cli(["help", "--quiet"])
    assert res_env.returncode == 3
    assert res_env.stdout == ""
    assert res_env.stderr == ""


def test_adr_debug_overrides_no_pretty() -> None:
    """ADR: --debug must force pretty-printing, overriding --no-pretty."""
    res = run_cli(["help", "--debug", "--no-pretty", "--format", "json"])
    assert res.returncode == 0
    assert "\n" in res.stdout
    payload = assert_json_obj(res.stdout)
    assert "platform" in payload


def test_adr_debug_with_error_is_verbose_and_pretty() -> None:
    """ADR: An error under --debug must be verbose and pretty-printed."""
    res = run_cli(["help", "nonexistent", "--debug", "--no-pretty", "--format", "json"])
    assert res.returncode == 2

    output = res.stderr or res.stdout
    payload = extract_first_json(output)

    assert "error" in payload
    assert "platform" in payload
    assert payload["code"] == 2


def extract_first_json(text: str) -> dict[str, Any]:
    """Extract first {...} block from text (non-recursive, works for flat JSON)."""
    m = re.search(r"(\{.*?\})", text, re.DOTALL)
    if not m:
        raise AssertionError("No JSON object found in output:\n" + text)
    return cast(dict[str, Any], json.loads(m.group(1)))


@pytest.mark.parametrize(
    ("argv", "expected_msg"),
    [
        (["help", "--format"], "requires an argument"),
        (["help", "-f"], "requires an argument"),
    ],
)
def test_adr_format_errors_and_contract(argv: list[str], expected_msg: str) -> None:
    """ADR-0002.4: --format with a missing argument must produce a standard error payload."""
    res = run_cli(argv)
    assert res.returncode == 2
    payload = assert_json_obj(res.stdout)
    assert "error" in payload
    assert "code" in payload
    assert payload["code"] == 2
    assert expected_msg in payload["error"].lower()


@given(
    flags=st.lists(
        st.sampled_from(KNOWN_FLAGS + ["--unknownflag", "-x"]),
        min_size=1,
        max_size=4,
        unique=True,
    ),
    value=st.one_of(st.just("json"), st.just("yaml"), st.just("bogus"), st.none()),
)
@settings(suppress_health_check=[HealthCheck.too_slow], max_examples=50, deadline=None)
def test_help_fuzz_flags(flags: list[str], value: str | None) -> None:
    """Fuzz various flag combinations for the help command, respecting ADR."""
    args: list[str] = []
    has_help = "-h" in flags or "--help" in flags
    has_quiet = "-q" in flags or "--quiet" in flags
    has_format = "-f" in flags or "--format" in flags
    has_unknown = "--unknownflag" in flags or "-x" in flags

    for f in flags:
        args.append(f)
        if f in ("--format", "-f") and value is not None:
            args.append(value)

    res = run_cli(["help", *args])

    if has_help:
        assert res.returncode == 0
        assert "usage:" in res.stdout.lower()
        return

    is_format_error = has_format and value is None
    is_bogus_format = has_format and value == "bogus"

    if is_format_error or is_bogus_format or has_unknown:
        assert res.returncode == 2
        if has_quiet:
            assert res.stdout == ""
            assert res.stderr == ""
        else:
            assert "error" in (res.stdout + res.stderr).lower()
    else:
        assert res.returncode == 0
        if has_quiet:
            assert res.stdout == ""
            assert res.stderr == ""
