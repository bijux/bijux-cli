# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for Bijux CLI.

Tests invoke the built CLI binary via subprocess, simulating real user interactions
with the installed wheel file, covering all commands, options, and edge cases.
"""

from __future__ import annotations

from collections.abc import Callable, Iterable, Iterator
import json
import os
from pathlib import Path
import re
import shlex
import shutil
from subprocess import CompletedProcess, TimeoutExpired, run
import sys
from typing import Any, cast

import pexpect  # type: ignore[import-untyped]
import pytest
import yaml  # pyright: ignore[reportMissingModuleSource]

ROOT = Path(__file__).resolve().parent.parent.parent
_ANSI_RE = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]")


def find_bijux_binary() -> Path:
    """Locate the bijux executable for end-to-end tests.

    Searches for the binary in a specific order:
    1. BIJUX_BIN environment variable.
    2. Sibling to the current Python interpreter (e.g., in a .venv).
    3. Any .tox directory.
    4. The system PATH.
    5. A local ./bin directory as a fallback.

    Raises:
        FileNotFoundError: If the bijux binary cannot be found in any of the
            searched locations.
    """
    exe_name = "bijux.exe" if os.name == "nt" else "bijux"

    if (override := os.getenv("BIJUX_BIN")) and Path(override).is_file():
        return Path(override).resolve()

    sibling = Path(sys.executable).with_name(exe_name)
    if sibling.exists():
        return sibling

    for p in ROOT.glob(f".tox/*/*/{exe_name}"):
        if p.is_file():
            return p.resolve()

    if which := shutil.which("bijux"):
        return Path(which).resolve()

    local = ROOT / ("Scripts" if os.name == "nt" else "bin") / exe_name
    if local.exists():
        return local.resolve()

    raise FileNotFoundError(
        "Could not locate the 'bijux' binary in "
        "$BIJUX_BIN, interpreter venv, tox envs, PATH, or project ./bin."
    )


PYTHON = sys.executable
BIN = find_bijux_binary()
PROMPT_REGEX = re.compile(
    r"^(?:\x1b\[[\d;?]*[A-Za-z])*bijux>\s*(?:\x1b\[[\d;?]*[A-Za-z])*\s*$", re.MULTILINE
)
TEST_TEMPLATE = str((Path(__file__).parent.parent.parent / "plugin_template").resolve())
_JSON_RE = re.compile(r"\{.*\}")

_bin = shutil.which("bijux")
_fallback_cmd = [sys.executable, "-m", "bijux_cli"] if _bin is None else [str(_bin)]

_repo_root = Path(__file__).resolve().parents[2]


def _unique_pathlist(*segments: str) -> str:
    """Create a unique, ordered, path-separated string from segments.

    Args:
        *segments: A variable number of path strings to join.

    Returns:
        An `os.pathsep`-joined string with duplicates removed.
    """
    seen: set[str] = set()
    uniq: list[str] = []
    for seg in segments:
        for part in seg.split(os.pathsep):
            if part and part not in seen:
                seen.add(part)
                uniq.append(part)
    return os.pathsep.join(uniq)


def run_cli(
    args: list[str] | str,
    *,
    env: dict[str, str] | None = None,
    input_data: str | None = None,
    timeout: int = 10,
) -> CompletedProcess[str]:
    """Launch the Bijux CLI in a subprocess.

    This function mimics a user invoking the installed binary, providing a
    robust way to run end-to-end tests. It sets up a standard test
    environment and captures the output.

    Args:
        args: A list of command-line arguments or a single shell-style string.
        env: An optional dictionary of environment variables to set.
        input_data: Optional string to pass to the process's stdin.
        timeout: The timeout in seconds for the command.

    Returns:
        A `subprocess.CompletedProcess` instance containing the results.
        If a timeout occurs, a `CompletedProcess` is still returned with a
        return code of 124.
    """
    if isinstance(args, str):
        args = shlex.split(args)

    merged = os.environ.copy()
    merged.update(env or {})

    merged["PYTHONIOENCODING"] = "utf-8"
    merged["BIJUXCLI_TEST_MODE"] = "1"
    merged["BIJUXCLI_BIN"] = _fallback_cmd[0]
    merged.pop("VERBOSE_DI", None)

    merged["PYTHONPATH"] = _unique_pathlist(
        str(_repo_root), merged.get("PYTHONPATH", "")
    )

    cmd = [*_fallback_cmd, *args]

    try:
        return run(  # noqa: S603
            cmd,
            input=input_data,
            text=True,
            capture_output=True,
            env=merged,
            timeout=timeout,
            start_new_session=True,
        )
    except TimeoutExpired as exc:
        stdout = (
            exc.stdout.decode("utf-8", "replace")
            if isinstance(exc.stdout, bytes)
            else exc.stdout or ""
        )
        stderr = (
            exc.stderr.decode("utf-8", "replace")
            if isinstance(exc.stderr, bytes)
            else exc.stderr or ""
        )
        return CompletedProcess(
            cmd,
            returncode=124,
            stdout=stdout,
            stderr=stderr + f"\n[TIMEOUT after {timeout}s]",
        )


def _decolorise(text: str) -> str:
    """Remove ANSI color and style escape codes from a string.

    Args:
        text: The string to process.

    Returns:
        The string with ANSI codes removed.
    """
    return _ANSI_RE.sub("", text)


def assert_text(res: str | CompletedProcess[str], content: str) -> None:
    """Assert that a specific text fragment exists in the command's output.

    Args:
        res: The result from `run_cli` or a raw string.
        content: The text fragment to search for, case-insensitively.
    """
    if isinstance(res, str):
        full_text_for_error = res
    else:
        full_text_for_error = (res.stdout or "") + (res.stderr or "")

    text_plain = _decolorise(full_text_for_error).lower()
    assert content.lower() in text_plain, (
        f"""Expected text '{content}' not found in: '{full_text_for_error}'"""
    )


def find_json_objects(s: str) -> Iterator[str]:
    """Extract and yield all top-level JSON objects from a string.

    This function handles nested structures and strings to correctly identify
    the boundaries of JSON objects.

    Args:
        s: The string to search through.

    Yields:
        Each valid, top-level JSON object found in the string.
    """
    i, n = 0, len(s)
    while i < n:
        if s[i] != "{":
            i += 1
            continue
        start, open_count, in_string, escaped = i, 1, False, False
        i += 1
        while i < n and open_count > 0:
            char = s[i]
            if in_string:
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    in_string = False
            else:
                if char == '"':
                    in_string = True
                elif char == "{":
                    open_count += 1
                elif char == "}":
                    open_count -= 1
            i += 1
        if open_count == 0:
            yield s[start:i]
        else:
            i = start + 1


def _try_parse_many(
    text: str, parsers: Iterable[Callable[[str], Any]]
) -> Iterable[tuple[dict[str, Any], Exception | None]]:
    """Attempt to parse text with multiple parsers, yielding results.

    Args:
        text: The text to parse.
        parsers: An iterable of parsing functions (e.g., `json.loads`).

    Yields:
        A tuple of `(result, error)` for each parser. If parsing succeeds,
        `error` is None. If it fails, `result` is an empty dict.
    """
    for parse in parsers:
        try:
            yield parse(text), None
        except Exception as exc:
            yield {}, exc


def _match(
    obj: dict[str, Any],
    key: str,
    expected: Any | None,
) -> tuple[bool, bool]:
    """Check if a dictionary contains a key with an expected value.

    Args:
        obj: The dictionary to check.
        key: The key to look for.
        expected: The expected value. If None, only checks for key presence.

    Returns:
        A tuple of (ok, mismatch), where `ok` is True if the value matches
        and `mismatch` is True if the types differ but string representations match.
    """
    if key not in obj:
        return False, False
    actual = obj[key]
    if expected is None or actual == expected:
        return True, False
    return False, str(actual) == str(expected)


def assert_log_has(
    proc_or_str: str | CompletedProcess[str],
    key: str,
    value: Any | None = None,
) -> None:
    """Assert that a key (and optionally value) appears in the CLI output.

    This helper robustly checks for a key-value pair by parsing the entire
    output stream, individual JSON objects, and line-by-line content.

    Args:
        proc_or_str: The result from `run_cli` or a raw string.
        key: The key to find in the structured output.
        value: The optional expected value for the key.
    """
    if isinstance(proc_or_str, CompletedProcess):
        stream = (proc_or_str.stdout or "") + (proc_or_str.stderr or "")
    else:
        stream = str(proc_or_str)

    stream_plain = _decolorise(stream)
    parsers: list[Callable[[str], Any]] = [json.loads, yaml.safe_load]
    saw_type_mismatch, last_bad = False, {}

    for obj, _ in _try_parse_many(stream_plain, parsers):
        ok, mismatch = _match(obj, key, value)
        if ok:
            return
        if mismatch:
            raise AssertionError(
                f"Key '{key}' found but value type mismatch: {obj[key]!r} vs {value!r}"
            )
    for block in find_json_objects(stream_plain):
        for obj, _ in _try_parse_many(block, [json.loads]):
            ok, mismatch = _match(obj, key, value)
            if ok:
                return
            if mismatch:
                saw_type_mismatch, last_bad = True, obj
    for line in stream_plain.splitlines():
        candidates = [line]
        if "output=" in line:
            candidates.append(line.split("output=", 1)[1].strip())
        for cand in candidates:
            for obj, _ in _try_parse_many(cand, parsers):
                ok, mismatch = _match(obj, key, value)
                if ok:
                    return
                if mismatch:
                    saw_type_mismatch, last_bad = True, obj
    if saw_type_mismatch:
        raise AssertionError(
            f"Key '{key}' found but value type mismatch: {last_bad.get(key)!r} vs {value!r}"
        )
    raise AssertionError(f"Key '{key}' with value '{value}' not found in output.")


def extract_json_fragments(text: str) -> Iterator[str]:
    """Yield JSON substrings from mixed text, robust to prefixes.

    Args:
        text: The string to search for JSON fragments.

    Yields:
        Each valid JSON object found as a string.
    """
    json_pat = re.compile(r"(\{.*?\})")
    for line in text.splitlines():
        for match in json_pat.finditer(line):
            yield match.group(1)


def last_json_with(stdout: str, *required_keys: str) -> dict[str, Any]:
    """Find the last JSON object in text that contains all required keys.

    This function robustly parses a string that may contain multiple JSON
    objects mixed with other text (like log lines) and returns the last
    valid object that includes all the specified keys.

    Args:
        stdout: The string containing mixed text and JSON to parse.
        *required_keys: A variable number of string keys that must be present
            in the returned dictionary.

    Returns:
        The last matching dictionary found, or an empty dictionary if no
        match is found.
    """
    json_pattern = re.compile(r"\{.*?\}", re.DOTALL)
    matches = json_pattern.findall(stdout)
    for fragment in reversed(matches):
        try:
            obj = json.loads(fragment)
            if isinstance(obj, dict) and all(k in obj for k in required_keys):
                return cast(dict[str, Any], obj)
        except Exception:  # noqa: S112
            continue
    return cast(dict[str, Any], {})


def spawn_repl(
    env: dict[str, str], extra_args: list[str] | None = None, timeout: int = 5
) -> pexpect.spawn[str]:
    """Spawn a REPL process for interactive testing.

    Args:
        env: A dictionary of environment variables for the process.
        extra_args: A list of extra arguments to pass to the REPL command.
        timeout: The timeout in seconds for pexpect operations.

    Returns:
        A `pexpect.spawn` instance connected to the REPL process.
    """
    cmd = [str(BIN), "repl", *(extra_args or [])]
    return pexpect.spawn(
        cmd[0],
        cmd[1:],
        env=env,  # pyright: ignore[reportArgumentType]
        encoding="utf-8",
        timeout=timeout,
    )


@pytest.fixture
def bijux_env(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> dict[str, str]:
    """Create a standard test environment for Bijux CLI.

    This pytest fixture sets up a temporary config file and standard
    environment variables used across the e2e test suite.

    Args:
        tmp_path: The pytest `tmp_path` fixture for creating temporary files.
        monkeypatch: The pytest `monkeypatch` fixture for modifying the environment.

    Returns:
        A copy of the configured `os.environ` dictionary.
    """
    config_file = tmp_path / ".env"
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(config_file))
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "1")
    return os.environ.copy()


def parse_json_lines(out: str) -> list[dict[str, Any]]:
    """Parse a stream of text containing one JSON object per line.

    Args:
        out: The string output from a CLI command.

    Returns:
        A list of parsed dictionary objects.
    """
    objs = []
    for line in out.splitlines():
        line = line.strip()
        try:
            objs.append(json.loads(line))
        except Exception:  # noqa: S112
            continue
    return objs


def normalize_history_payload(payload: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Normalize a history payload for consistent comparison.

    This removes the 'timestamp' field and sorts the entries to allow for
    deterministic comparisons between test runs.

    Args:
        payload: A list of history entry dictionaries.

    Returns:
        A normalized and sorted list of history entries.
    """
    return sorted(
        [{k: v for k, v in entry.items() if k != "timestamp"} for entry in payload],
        key=lambda d: d.get("command", ""),
    )
