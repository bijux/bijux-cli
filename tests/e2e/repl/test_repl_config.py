# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi
# ruff: noqa: S101

"""End-to-end tests for `bijux repl` config."""

from __future__ import annotations

from collections.abc import Iterable
import json
from pathlib import Path
import re
from typing import Any

import pytest

from tests.e2e.conftest import run_cli


def _json_lines(stdout: str) -> Iterable[dict[str, Any]]:
    """Yield JSON objects from stdout, handling multi-line JSON."""
    json_pattern = re.compile(r"\{(?:[^{}]|\{[^{}]*\})*\}")
    matches = json_pattern.finditer(stdout)

    for match in matches:
        json_str = match.group(0).strip()
        if not json_str:
            continue
        try:
            obj = json.loads(json_str)
            if isinstance(obj, dict):
                yield obj
        except json.JSONDecodeError:
            continue


def _find_line_with(
    data: Iterable[dict[str, Any]], *, key: str, value: Any = None
) -> dict[str, Any] | None:
    """Return first dict where key (and optional value) matches."""
    for obj in data:
        if key in obj and (value is None or obj[key] == value):
            return obj
    return None


@pytest.fixture
def env(tmp_path: Path) -> dict[str, str]:
    """Environment for REPL: isolated config, test mode."""
    return {
        "BIJUXCLI_CONFIG": str(tmp_path / ".env"),
        "BIJUXCLI_TEST_MODE": "1",
    }


def test_config_set_and_get_returns_value(env: dict[str, str]) -> None:
    """Verify a config value can be set and then retrieved."""
    out = run_cli(
        ["repl"],
        env=env,
        input_data="config set foo=bar\nconfig get foo\nexit\n",
    ).stdout
    objs = list(_json_lines(out))
    assert _find_line_with(objs, key="key", value="foo")
    assert _find_line_with(objs, key="value", value="bar")


def test_config_persists_across_sessions(env: dict[str, str]) -> None:
    """Ensure config values are saved and available in a new session."""
    run_cli(["repl"], env=env, input_data="config set alpha=1\nexit\n")
    out = run_cli(["repl"], env=env, input_data="config get alpha\nexit\n").stdout
    objs = list(_json_lines(out))
    assert _find_line_with(objs, key="value", value="1")


def test_config_unset_removes_key(env: dict[str, str]) -> None:
    """Check that 'unset' correctly removes a configuration key."""
    run_cli(["repl"], env=env, input_data="config set x=42\nexit\n")
    run_cli(["repl"], env=env, input_data="config unset x\nexit\n")
    out = run_cli(["repl"], env=env, input_data="config get x\nexit\n").stderr
    objs = list(_json_lines(out))
    err = _find_line_with(objs, key="error")
    assert err
    assert "not found" in err["error"].lower()


def test_config_clear_removes_all(env: dict[str, str]) -> None:
    """Verify 'clear' command removes all configuration keys."""
    input_data = "config set a=1\nconfig set b=2\nconfig clear\nconfig get a\nconfig get b\nexit\n"
    out = run_cli(["repl"], env=env, input_data=input_data).stderr
    errs = [o for o in _json_lines(out) if "error" in o]
    assert len(errs) >= 2


def test_config_set_allows_multiple_equals(env: dict[str, str]) -> None:
    """Test setting a value that contains an equals sign."""
    out = run_cli(
        ["repl"],
        env=env,
        input_data="config set key=v=extra\nconfig get key\nexit\n",
    ).stdout
    vals = [o["value"] for o in _json_lines(out) if "value" in o]
    assert "v=extra" in vals


def test_config_set_empty_value(env: dict[str, str]) -> None:
    """Ensure setting a key with an empty value works correctly."""
    out = run_cli(
        ["repl"],
        env=env,
        input_data="config set empty=\nconfig get empty\nexit\n",
    ).stdout
    objs = list(_json_lines(out))
    assert _find_line_with(objs, key="key", value="empty")


def test_config_unset_nonexistent_reports_error(env: dict[str, str]) -> None:
    """Verify unsetting a key that doesn't exist reports an error."""
    out = run_cli(["repl"], env=env, input_data="config unset none\nexit\n").stderr
    assert _find_line_with(_json_lines(out), key="error")


def test_config_get_nonexistent_reports_error(env: dict[str, str]) -> None:
    """Verify getting a key that doesn't exist reports an error."""
    out = run_cli(["repl"], env=env, input_data="config get missing\nexit\n").stderr
    assert _find_line_with(_json_lines(out), key="error")


def test_config_load_nonexistent_file_reports_error(
    env: dict[str, str], tmp_path: Path
) -> None:
    """Check that loading from a non-existent file reports an error."""
    missing = tmp_path / "nope.json"
    out = run_cli(["repl"], env=env, input_data=f"config load {missing}\nexit\n").stderr
    assert _find_line_with(_json_lines(out), key="error")


def test_config_export_creates_file(env: dict[str, str], tmp_path: Path) -> None:
    """Ensure 'export' command creates the specified output file."""
    target = tmp_path / "out.env"
    run_cli(["repl"], env=env, input_data="config set m=100\nexit\n")
    run_cli(["repl"], env=env, input_data=f"config export {target}\nexit\n")
    assert target.exists()
    assert "BIJUXCLI_M=100" in target.read_text()


def test_config_export_overwrites_file(env: dict[str, str], tmp_path: Path) -> None:
    """Verify that 'export' overwrites an existing file."""
    target = tmp_path / "out.env"
    target.write_text("stale")
    run_cli(["repl"], env=env, input_data="config set z=9\nexit\n")
    run_cli(["repl"], env=env, input_data=f"config export {target}\nexit\n")
    assert "BIJUXCLI_Z=9" in target.read_text()


def test_export_permission_denied_reports_error(
    env: dict[str, str], tmp_path: Path
) -> None:
    """Test that exporting to a read-only file reports a permission error."""
    target = tmp_path / "locked.env"
    target.write_text("stay")
    target.chmod(0o400)
    out = run_cli(
        ["repl"], env=env, input_data=f"config export {target}\nexit\n"
    ).stderr
    assert _find_line_with(_json_lines(out), key="error")
    target.chmod(0o600)


def test_config_load_restores_values(env: dict[str, str], tmp_path: Path) -> None:
    """Check that 'load' correctly restores config values from a file."""
    target = tmp_path / "cfg.env"
    run_cli(["repl"], env=env, input_data="config set p=7\nexit\n")
    run_cli(["repl"], env=env, input_data=f"config export {target}\nexit\n")
    run_cli(["repl"], env=env, input_data="config clear\nexit\n")
    out = run_cli(
        ["repl"],
        env=env,
        input_data=f"config load {target}\nconfig get p\nexit\n",
    ).stdout
    assert _find_line_with(_json_lines(out), key="value", value="7")


def test_config_list_shows_keys(env: dict[str, str]) -> None:
    """Verify the 'list' command displays all configured keys."""
    run_cli(["repl"], env=env, input_data="config set a=1\nconfig set b=2\nexit\n")
    out = run_cli(["repl"], env=env, input_data="config list\nexit\n").stdout
    items = _find_line_with(_json_lines(out), key="items")
    assert items is not None
    assert isinstance(items["items"], list)
    keys = {d["key"].lower() for d in items["items"]}
    assert {"a", "b"} <= keys


def test_config_list_empty_after_clear(env: dict[str, str]) -> None:
    """Ensure 'list' shows an empty set after 'clear' is used."""
    run_cli(["repl"], env=env, input_data="config set x=1\nconfig clear\nexit\n")
    out = run_cli(["repl"], env=env, input_data="config list\nexit\n").stdout
    items = _find_line_with(_json_lines(out), key="items")
    assert items
    assert items["items"] == []


@pytest.mark.parametrize(
    ("cmd", "expected_error"),
    [
        ("config set", "key=value"),
        ("config get", "key"),
        ("config unset", "key"),
    ],
)
def test_missing_args_show_usage_or_error(
    env: dict[str, str], cmd: str, expected_error: str
) -> None:
    """Test that commands report an error when required arguments are missing."""
    out = run_cli(["repl"], env=env, input_data=f"{cmd}\nexit\n").stdout
    err = _find_line_with(_json_lines(out), key="error")
    assert err
    assert expected_error in err["error"].lower()


def test_config_set_invalid_key_format_reports_error(
    env: dict[str, str],
) -> None:
    """Ensure setting a key with invalid characters reports an error."""
    out = run_cli(["repl"], env=env, input_data="config set 'bad key'=v\nexit\n").stderr
    assert _find_line_with(_json_lines(out), key="error")


def test_config_set_long_value(env: dict[str, str]) -> None:
    """Verify setting and getting a very long value works correctly."""
    big = "X" * 5000
    out = run_cli(
        ["repl"],
        env=env,
        input_data=f"config set long={big}\nconfig get long\nexit\n",
    ).stdout
    assert _find_line_with(_json_lines(out), key="value", value=big)


def test_config_reload_picks_up_external_change(
    env: dict[str, str], tmp_path: Path
) -> None:
    """Test that 'reload' picks up changes made directly to the config file."""
    cfg = Path(env["BIJUXCLI_CONFIG"])
    run_cli(["repl"], env=env, input_data="config set k=1\nexit\n")
    cfg.write_text("BIJUXCLI_K=2\n")
    out = run_cli(
        ["repl"],
        env=env,
        input_data="config reload\nconfig get k\nexit\n",
    ).stdout
    assert _find_line_with(_json_lines(out), key="value", value="2")


def test_config_reload_deletes_key(env: dict[str, str], tmp_path: Path) -> None:
    """Test 'reload' when a key is removed from the config file externally."""
    cfg = Path(env["BIJUXCLI_CONFIG"])
    run_cli(["repl"], env=env, input_data="config set k=1\nexit\n")
    cfg.write_text("")
    out = run_cli(
        ["repl"],
        env=env,
        input_data="config reload\nconfig get k\nexit\n",
    ).stderr
    err = _find_line_with(_json_lines(out), key="error")
    assert err
    assert "not found" in err["error"].lower()


def test_config_semicolon_chaining(env: dict[str, str]) -> None:
    """Verify multiple commands can be chained on one line with semicolons."""
    out = run_cli(
        ["repl"],
        env=env,
        input_data="config set u=10; config get u; config unset u; config get u; exit\n",
    )
    vals = [o.get("value") for o in _json_lines(out.stdout) if "value" in o]
    errs = [o for o in _json_lines(out.stderr) if "error" in o]
    assert "10" in vals
    assert errs


def test_config_comments_and_blank_lines_ignored(env: dict[str, str]) -> None:
    """Ensure that comments and blank lines in scripts are ignored."""
    script = "#comment\n\nconfig set a=5\n#skip\nconfig get a\nexit\n"
    out = run_cli(["repl"], env=env, input_data=script).stdout
    assert _find_line_with(_json_lines(out), key="value", value="5")


def test_config_many_set_unset_cycle(env: dict[str, str]) -> None:
    """Test a rapid sequence of setting and unsetting many keys."""
    script = (
        "\n".join(f"config set k{i}=v{i}" for i in range(10))
        + "\n"
        + "\n".join(f"config unset k{i}" for i in range(10))
        + "\n"
        + "\n".join(f"config get k{i}" for i in range(10))
        + "\n"
        + "\nexit\n"
    )
    out = run_cli(["repl"], env=env, input_data=script).stderr
    errs = [o for o in _json_lines(out) if "error" in o]
    assert len(errs) >= 10


def test_config_json_like_value(env: dict[str, str]) -> None:
    """Check setting a value that is a JSON-formatted string."""
    val = '{"a":1,"b":2}'
    out = run_cli(
        ["repl"],
        env=env,
        input_data='config set jsn="{\\"a\\":1,\\"b\\":2}"\nconfig get jsn\nexit\n',
    ).stdout
    assert _find_line_with(_json_lines(out), key="value", value=val)


def test_config_list_format_structure(env: dict[str, str]) -> None:
    """Confirm the structure of the 'list' command's JSON output."""
    run_cli(["repl"], env=env, input_data="config set one=1\nexit\n")
    out = run_cli(["repl"], env=env, input_data="config list\nexit\n").stdout
    item = _find_line_with(_json_lines(out), key="items")
    assert item is not None
    assert isinstance(item["items"], list)
