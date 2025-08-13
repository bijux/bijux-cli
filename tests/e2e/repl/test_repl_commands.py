# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for `bijux repl` commands."""

from __future__ import annotations

from collections.abc import Callable
import json
from pathlib import Path
from typing import Any, Protocol

import pytest

from tests.e2e.conftest import assert_log_has, run_cli


class _ProcLike(Protocol):
    stdout: str | None
    stderr: str | None


def assert_log_has_any_output(
    proc: _ProcLike, key: str, value: str | None = None
) -> None:
    """Assert that key (and optional value) appears in either stdout or stderr."""
    for stream in (proc.stdout, proc.stderr):
        if stream and (key in stream) and (value is None or value in stream):
            return
    raise AssertionError(
        f"Key {key!r} with value {value!r} not found in stdout or stderr.\n"
        f"stdout={proc.stdout!r}\nstderr={proc.stderr!r}"
    )


@pytest.fixture
def env(tmp_path: Path) -> dict[str, str]:
    """Isolated REPL environment: per-test config file plus TEST_MODE."""
    return {
        "BIJUXCLI_CONFIG": str(tmp_path / ".env"),
        "BIJUXCLI_TEST_MODE": "1",
    }


@pytest.mark.parametrize("i", range(50))
def test_e2e_repl_command_config_set_get_variations(
    env: dict[str, str], tmp_path: Path, i: int
) -> None:
    """Ensure the REPL sets and retrieves config values correctly across 50 distinct key/value pairs."""
    key = f"key{i}"
    val = f"val{i}"
    script = f"config set {key}={val}\nconfig get {key}\nexit\n"
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", val)


@pytest.mark.parametrize("key", [f"missing{i}" for i in range(10)])
def test_e2e_repl_command_config_unset_nonexistent(
    env: dict[str, str], key: str
) -> None:
    """Test that unsetting nonexistent config keys returns an error as expected."""
    res = run_cli(["repl"], env=env, input_data=f"config unset {key}\nexit\n")
    assert res.returncode == 0
    assert_log_has(res.stderr, "error")


@pytest.mark.parametrize("n", range(10))
def test_e2e_repl_command_config_clear_and_list(
    env: dict[str, str], tmp_path: Path, n: int
) -> None:
    """Verify that clearing the config removes all entries and results in an empty config list."""
    sets = "".join(f"config set k{n}_{j}=v{j}\n" for j in range(n))
    script = f"{sets}config clear\nconfig list\nexit\n"
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0
    assert '"items":[]' in res.stdout.replace(" ", "")


BASIC_COMMANDS: list[tuple[str, Callable[[str], bool], str]] = [
    ("version\nexit\n", lambda o: "version" in o.lower(), "text"),
    (
        "status\nexit\n",
        lambda o: "status" in o.lower() or ("status" in _safe_json_keys(o)),
        "text|json",
    ),
    ("help\nexit\n", lambda o: "usage" in o.lower(), "text"),
    (
        "help config\nexit\n",
        lambda o: "usage" in o.lower() and "config" in o.lower(),
        "text",
    ),
    ("docs\nexit\n", lambda o: "available" in o.lower(), "text"),
    ("docs version\nexit\n", lambda o: "version" in o.lower(), "text"),
    (
        "memory list\nexit\n",
        lambda o: "keys" in _safe_json(o),
        "json",
    ),
    (
        "plugins list\nexit\n",
        lambda o: "plugins" in _safe_json(o),
        "json",
    ),
    (
        "config list\nexit\n",
        lambda o: "items" in _safe_json(o) or "key" in _safe_json(o),
        "json",
    ),
    ("config --help\nexit\n", lambda o: "usage" in o.lower(), "text"),
    (
        "sleep\nexit\n",
        lambda o: o == "" or "--seconds" in _safe_stderr(),
        "empty|error",
    ),
    (
        "audit\nexit\n",
        lambda o: "status" in _safe_json(o),
        "json",
    ),
    (
        "doctor\nexit\n",
        lambda o: "status" in _safe_json(o),
        "json",
    ),
    (
        "dev\nexit\n",
        lambda o: "status" in _safe_json(o),
        "json",
    ),
    (
        "history\nexit\n",
        lambda o: o.strip() == "[]"
        or ("entries" in _safe_json(o) and isinstance(_safe_json(o)["entries"], list)),
        "json|empty",
    ),
    (
        "config\nexit\n",
        lambda o: o.strip() == "{}" or bool(_safe_json(o)),
        "json",
    ),
    (
        "plugins\nexit\n",
        lambda o: o == "",
        "json|empty",
    ),
]


def _safe_json(output: str) -> dict[str, Any]:
    """Safely parse a JSON string, returning an empty dict on failure.

    Args:
        output (str): The JSON-formatted string to parse.

    Returns:
        dict[str, Any]: The parsed JSON as a dictionary, or an empty dict if parsing fails.
    """
    try:
        result = json.loads(output)
        if isinstance(result, dict):
            return result
        return {}
    except Exception:
        return {}


def _safe_json_keys(output: str) -> list[str]:
    """Safely extract the top-level keys from a JSON string.

    Args:
        output (str): The JSON-formatted string to parse.

    Returns:
        list[str]: A list of keys if parsing succeeds, or an empty list on failure.
    """
    try:
        return list(json.loads(output).keys())
    except Exception:
        return []


def _safe_stderr() -> str:
    """Return an empty string as a safe default for standard error output.

    Returns:
        str: An empty string.
    """
    return ""


@pytest.mark.parametrize(("script", "check", "desc"), BASIC_COMMANDS)
def test_e2e_repl_command_basic(
    env: dict[str, str], script: str, check: Callable[[str], bool], desc: str
) -> None:
    """Run basic REPL scripts and assert output passes the provided check function."""
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0
    assert check(res.stdout), (
        f"Failed for {script!r} ({desc}):\nstdout={res.stdout!r}\nstderr={getattr(res, 'stderr', '')!r}"
    )


CHAIN_CASES: list[tuple[str, str | None, str | None]] = [
    ("version; exit\n", "version", None),
    ("config set a=1; config get a; exit\n", "value", "1"),
    ("config set x=foo; config unset x; config get x; exit\n", "error", None),
    ("help; help config; exit\n", None, "Usage"),
    ("docs; docs help; exit\n", None, "Available"),
    ("version; status; exit\n", "status", None),
    ("config set y=2; version; config get y; exit\n", "value", "2"),
    ("config set z=3; config list; exit\n", "items", None),
    ("memory set m hi; memory get m; exit\n", "value", "hi"),
    ("plugins list; exit\n", "plugins", None),
]


@pytest.mark.parametrize(("script", "key", "expected"), CHAIN_CASES)
def test_e2e_repl_command_chaining(
    env: dict[str, str], script: str, key: str | None, expected: str | None
) -> None:
    """Test command chaining in the REPL and validate key or expected output in results."""
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0
    if key is None and expected:
        assert expected in res.stdout
    elif expected is None:
        assert_log_has_any_output(res, key or "")  # type: ignore[arg-type]
    else:
        assert_log_has_any_output(res, key or "", expected)  # type: ignore[arg-type]


FLAG_COMBINATIONS: list[tuple[str, str | None, str | None, int, str]] = [
    ("version -q\nexit\n", None, None, 0, "quiet: suppress output"),
    ("version -d\nexit\n", "version", None, 0, "debug: should print version"),
    ("version -f json\nexit\n", "version", None, 0, "json output"),
    ("version -f yaml\nexit\n", "version", None, 0, "yaml output"),
    ("config set f=1 -q\nexit\n", None, None, 0, "config set quiet, no output"),
    ("config set f=1 -d\nexit\n", "status", "updated", 0, "config set debug output"),
    (
        "config set f=1 -f yaml\nexit\n",
        "status",
        "updated",
        0,
        "config set yaml output",
    ),
    (
        "config set f=1 -f json\nexit\n",
        "status",
        "updated",
        0,
        "config set json output",
    ),
    (
        "config set f=42\nconfig get f -f yaml\nexit\n",
        "value",
        "42",
        0,
        "get present key, yaml",
    ),
    (
        "config set f=42\nconfig get f -f json\nexit\n",
        "value",
        "42",
        0,
        "get present key, json",
    ),
    (
        "config set f=42\nconfig get f -d\nexit\n",
        "value",
        "42",
        0,
        "get present key, debug",
    ),
    (
        "config get missing_key -f yaml\nexit\n",
        "error",
        "not found",
        0,
        "get missing key, yaml",
    ),
    (
        "config get missing_key -f json\nexit\n",
        "error",
        "not found",
        0,
        "get missing key, json",
    ),
    (
        "config get missing_key -d\nexit\n",
        "error",
        "not found",
        0,
        "get missing key, debug",
    ),
    ("status -q\nexit\n", None, None, 0, "status quiet"),
    ("status -d\nexit\n", "status", None, 0, "status debug"),
    ("status -f json\nexit\n", "status", None, 0, "status json"),
    ("status -f yaml\nexit\n", "status", None, 0, "status yaml"),
]


@pytest.mark.parametrize(("script", "key", "expected", "rc", "desc"), FLAG_COMBINATIONS)
def test_e2e_repl_command_flag_combinations(
    env: dict[str, str],
    script: str,
    key: str | None,
    expected: str | None,
    rc: int,
    desc: str,
) -> None:
    """Test all relevant flag combinations for core commands."""
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == rc, f"{desc} (rc={res.returncode}, expected={rc})"
    combined_output = (res.stdout or "").lower() + (res.stderr or "").lower()
    if key is None:
        assert (res.stdout or "").strip() == "", f"{desc} (output: {res.stdout})"
    else:
        assert key in combined_output, (
            f"{desc} (missing key '{key}' in output: {combined_output})"
        )
        if expected:
            assert expected.lower() in combined_output, (
                f"{desc} (missing expected '{expected}' in output: {combined_output})"
            )


@pytest.mark.parametrize(
    "script",
    [
        "\n#nothing\nexit\n",
        "#hello\n\nversion\nexit\n",
        "   \nconfig set a=b\nexit\n",
        "#foo\n#bar\n#baz\nexit\n",
        "\n\nconfig get a\nexit\n",
        " \t  \nversion\nexit\n",
        "# multiple\n#comments\nversion\nexit\n",
        "\n#mix\nconfig list\nexit\n",
        "config set x=1\n#comment after\nconfig get x\nexit\n",
        "config set y=2\n\n# blank above\nconfig get y\nexit\n",
    ],
)
def test_e2e_repl_command_comments_and_blanks(env: dict[str, str], script: str) -> None:
    """Verify the REPL correctly handles comments and blank lines in input scripts."""
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0
    if "version" in script:
        assert_log_has(res.stdout, "version")
    else:
        assert res.returncode == 0


@pytest.mark.parametrize(
    ("line", "value", "expect_errors"),
    [
        ("config set q='single quote'\nconfig get q\nexit\n", "single quote", []),
        ('config set q="double quote"\nconfig get q\nexit\n', "double quote", []),
        ('config set j="{\\"a\\":1}"\nconfig get j\nexit\n', '{"a":1}', []),
        (
            "config set sem=1;2;3\nconfig get sem\nexit\n",
            "1",
            ["No such command '2'", "No such command '3'"],
        ),
        ("config set space=hello\\ world\nconfig get space\nexit\n", "hello world", []),
        (
            "config set nl='line1\nline2'\nconfig get nl\nexit\n",
            None,
            ["Config key not found: nl"],
        ),
        ("config set sc=var$val\nconfig get sc\nexit\n", "var$val", []),
        ("config set pct=100%25\nconfig get pct\nexit\n", "100%25", []),
        ("config set dash=one-two\nconfig get dash\nexit\n", "one-two", []),
        ("config set under=foo_bar\nconfig get under\nexit\n", "foo_bar", []),
    ],
)
def test_e2e_repl_command_quoting_and_special(
    env: dict[str, str], line: str, value: str | None, expect_errors: list[str]
) -> None:
    """Test REPL handling of quoting, special characters, and error messages in commands."""
    res = run_cli(["repl"], env=env, input_data=line)
    assert res.returncode == 0

    if value is not None:
        assert_log_has(res.stdout, "value", value)
    if expect_errors:
        output = (res.stdout or "") + (res.stderr or "")
        for msg in expect_errors:
            assert msg in output, (
                f"Missing expected error '{msg}' in output: {output!r}"
            )


@pytest.mark.parametrize(
    "size", [1000, 2000, 5000, 10000, 15000, 20000, 25000, 30000, 50000, 100000]
)
def test_e2e_repl_command_large_input_sizes(env: dict[str, str], size: int) -> None:
    """Verify REPL correctness with large input sizes for config set/get commands."""
    key = "big"
    val = "x" * size
    script = f"config set {key}={val}\nconfig get {key}\nexit\n"
    res = run_cli(["repl"], env=env, input_data=script)
    assert res.returncode == 0
    assert val[:100] in res.stdout


@pytest.mark.parametrize(
    ("cmd", "expect_error"),
    [
        ("foo", True),
        ("bar", True),
        ("config foo", True),
        ("versionx", True),
        ("statuz", True),
        ("memory foo", True),
        ("docs foo", False),
        ("plugins bar", True),
        ("helpx", True),
        ("exitx", True),
    ],
)
def test_e2e_repl_command_unknown_errors(
    env: dict[str, str], cmd: str, expect_error: bool
) -> None:
    """Test REPL responses for unknown commands and validate presence or absence of error output."""
    res = run_cli(["repl"], env=env, input_data=f"{cmd}\nexit\n")
    assert res.returncode in (0, 1, 2)
    out = res.stdout + res.stderr
    if expect_error:
        assert "error" in out.lower() or "no such command" in out.lower(), (
            f"{cmd} did not produce expected error output: {out!r}"
        )
    else:
        assert "error" not in out.lower()
        assert "no such command" not in out.lower()
