# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux memory` command."""

from __future__ import annotations

import json
from pathlib import Path
import signal
from subprocess import PIPE, Popen
import sys
import threading
import time
from typing import Any

import pytest
import yaml  # pyright: ignore[reportMissingModuleSource]

from .conftest import run_cli


def test_e2e_memory_status_json() -> None:
    """Test the root 'memory' command with default JSON output."""
    res = run_cli(["memory"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert data["status"] == "ok"


def test_e2e_memory_status_yaml() -> None:
    """Test the root 'memory' command with YAML output."""
    res = run_cli(["memory", "--format", "yaml"])
    assert res.returncode == 0
    data = yaml.safe_load(res.stdout)
    assert data["status"] == "ok"
    assert "message" in data


def test_e2e_memory_status_debug() -> None:
    """Test the root 'memory' command with the --debug flag."""
    res = run_cli(["memory", "--debug"])
    assert res.returncode == 0
    assert "ok" in res.stdout


def test_e2e_memory_set_and_get() -> None:
    """Test setting and then getting a value."""
    res1 = run_cli(["memory", "set", "foo", "bar"])
    assert res1.returncode == 0
    res2 = run_cli(["memory", "get", "foo"])
    assert res2.returncode == 0
    data = json.loads(res2.stdout)
    assert data["value"] == "bar"


def test_e2e_memory_set_and_get_yaml() -> None:
    """Test setting and getting a value with YAML format."""
    res1 = run_cli(["memory", "set", "ykey", "yval", "--format", "yaml"])
    assert res1.returncode == 0
    res2 = run_cli(["memory", "get", "ykey", "--format", "yaml"])
    assert res2.returncode == 0
    data = yaml.safe_load(res2.stdout)
    assert data["value"] == "yval"


def test_e2e_memory_get_nonexistent() -> None:
    """Test that getting a non-existent key fails gracefully."""
    res = run_cli(["memory", "get", "notfound"])
    assert res.returncode != 0 or "null" in res.stdout or "None" in res.stdout


def test_e2e_memory_clear_and_get() -> None:
    """Test getting a key after the memory has been cleared."""
    run_cli(["memory", "set", "to_be_cleared", "gone"])
    res_clear = run_cli(["memory", "clear"])
    assert res_clear.returncode == 0
    res_get = run_cli(["memory", "get", "to_be_cleared"])
    assert (
        res_get.returncode != 0 or "null" in res_get.stdout or "None" in res_get.stdout
    )


def test_e2e_memory_set_debug_mode() -> None:
    """Test setting a value with the --debug flag."""
    res = run_cli(["memory", "set", "dkey", "dval", "--debug"])
    assert res.returncode == 0
    assert "dkey" in res.stdout
    assert "dval" in res.stdout


def test_e2e_memory_set_invalid_format() -> None:
    """Test that using an invalid format fails."""
    res = run_cli(["memory", "set", "fkey", "fval", "--format", "invalid"])
    assert res.returncode != 0
    msg = (res.stdout or "") + (res.stderr or "")
    msg = msg.lower()
    assert (
        "unsupported format" in msg
        or "not a valid" in msg
        or "invalid format" in msg
        or "invalid" in msg
    ), msg


def test_e2e_memory_quiet_mode_suppresses_output() -> None:
    """Test that the --quiet flag suppresses output."""
    res = run_cli(["memory", "--quiet"])
    assert res.returncode == 0
    assert res.stdout.strip() == "" or res.stdout.strip() == "{}"


def test_e2e_memory_set_empty_value() -> None:
    """Test setting a key to an empty string value."""
    res = run_cli(["memory", "set", "emptykey", ""])
    assert res.returncode == 0
    getres = run_cli(["memory", "get", "emptykey"])
    data = json.loads(getres.stdout)
    assert data["value"] == ""


def test_e2e_memory_overwrite_value() -> None:
    """Test that setting an existing key overwrites its value."""
    run_cli(["memory", "set", "overwrite", "first"])
    run_cli(["memory", "set", "overwrite", "second"])
    res = run_cli(["memory", "get", "overwrite"])
    data = json.loads(res.stdout)
    assert data["value"] == "second"


def test_e2e_memory_delete_key() -> None:
    """Test deleting a key."""
    run_cli(["memory", "set", "delkey", "toremove"])
    delres = run_cli(["memory", "delete", "delkey"])
    assert delres.returncode == 0
    getres = run_cli(["memory", "get", "delkey"])
    assert getres.returncode != 0 or "null" in getres.stdout or "None" in getres.stdout


def test_e2e_memory_list_keys() -> None:
    """Test listing all stored keys."""
    run_cli(["memory", "set", "key1", "val1"])
    run_cli(["memory", "set", "key2", "val2"])
    res = run_cli(["memory", "list", "--format", "json"])
    data = json.loads(res.stdout)
    assert "key1" in data["keys"]
    assert "key2" in data["keys"]


def test_e2e_memory_list_empty() -> None:
    """Test listing keys when the store is empty."""
    run_cli(["memory", "clear"])
    res = run_cli(["memory", "list", "--format", "json"])
    data = json.loads(res.stdout)
    assert data["keys"] == []


def test_e2e_memory_set_unicode_key_value() -> None:
    """Test setting and getting a key and value with unicode characters."""
    res = run_cli(["memory", "set", "üñîçødë", "✓"])
    assert res.returncode == 0
    getres = run_cli(["memory", "get", "üñîçødë"])
    data = json.loads(getres.stdout)
    assert data["value"] == "✓"


def test_e2e_memory_set_large_value() -> None:
    """Test setting and getting a very large value."""
    large = "x" * 10_000
    res = run_cli(["memory", "set", "bigkey", large])
    assert res.returncode == 0
    getres = run_cli(["memory", "get", "bigkey"])
    data = json.loads(getres.stdout)
    assert data["value"] == large


def test_e2e_memory_concurrent_set_and_get(tmp_path: Path) -> None:
    """Test concurrent read and write operations."""

    def set_key(val: str) -> None:
        """Executes the CLI command to set the 'conc' key to a given value."""
        run_cli(["memory", "set", "conc", val])

    threads = [threading.Thread(target=set_key, args=(str(i),)) for i in range(5)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    res = run_cli(["memory", "get", "conc"])
    assert res.returncode == 0
    val = json.loads(res.stdout)["value"]
    assert val in {str(i) for i in range(5)}


def test_e2e_memory_help() -> None:
    """Test that the --help flag works."""
    res = run_cli(["memory", "--help"])
    assert res.returncode == 0
    assert "Usage" in res.stdout or "help" in res.stdout


def test_e2e_memory_list_yaml_format() -> None:
    """Test listing keys with YAML format."""
    run_cli(["memory", "set", "ky", "vl"])
    res = run_cli(["memory", "list", "--format", "yaml"])
    assert res.returncode == 0
    data = yaml.safe_load(res.stdout)
    assert "ky" in data["keys"]


def test_e2e_memory_delete_nonexistent_key() -> None:
    """Test that deleting a non-existent key fails."""
    res = run_cli(["memory", "delete", "does_not_exist"])
    assert res.returncode != 0
    assert "not found" in res.stderr.lower() or "error" in res.stderr.lower()


def test_e2e_memory_list_after_delete() -> None:
    """Test that a deleted key does not appear when listing keys."""
    run_cli(["memory", "set", "foo1", "bar1"])
    run_cli(["memory", "delete", "foo1"])
    res = run_cli(["memory", "list"])
    data = json.loads(res.stdout)
    assert "foo1" not in data["keys"]


def test_e2e_memory_set_and_get_multiple_keys() -> None:
    """Test setting and getting multiple keys in sequence."""
    pairs = [("a", "1"), ("b", "2"), ("c", "3")]
    for k, v in pairs:
        run_cli(["memory", "set", k, v])
    for k, v in pairs:
        res = run_cli(["memory", "get", k])
        assert json.loads(res.stdout)["value"] == v


def test_e2e_memory_delete_then_set_again() -> None:
    """Test that a key can be set again after being deleted."""
    run_cli(["memory", "set", "x", "y"])
    run_cli(["memory", "delete", "x"])
    res1 = run_cli(["memory", "set", "x", "z"])
    assert res1.returncode == 0
    res2 = run_cli(["memory", "get", "x"])
    assert json.loads(res2.stdout)["value"] == "z"


def test_e2e_memory_set_special_chars_key() -> None:
    """Test setting and getting a key with special characters."""
    res = run_cli(["memory", "set", "!@#$%^&*()", "val"])
    assert res.returncode == 0
    getres = run_cli(["memory", "get", "!@#$%^&*()"])
    assert json.loads(getres.stdout)["value"] == "val"


def test_e2e_memory_set_special_chars_value() -> None:
    """Test setting and getting a value with special characters."""
    value = "!@#$%^&*()_+-={}[]:\";'<>?,./"
    run_cli(["memory", "set", "specialval", value])
    res = run_cli(["memory", "get", "specialval"])
    assert json.loads(res.stdout)["value"] == value


def test_e2e_memory_set_multiline_value() -> None:
    """Test setting and getting a multiline value."""
    val = "line1\nline2\nline3"
    run_cli(["memory", "set", "multiline", val])
    res = run_cli(["memory", "get", "multiline"])
    assert json.loads(res.stdout)["value"] == val


def test_e2e_memory_list_returns_all_keys_after_clear_and_set() -> None:
    """Test that list shows all keys after a clear and set cycle."""
    run_cli(["memory", "clear"])
    keys = ["one", "two", "three"]
    for k in keys:
        run_cli(["memory", "set", k, "x"])
    res = run_cli(["memory", "list"])
    d = json.loads(res.stdout)
    for k in keys:
        assert k in d["keys"]


def test_e2e_memory_set_large_key() -> None:
    """Test setting and getting a very large key."""
    key = "k" * 2048
    run_cli(["memory", "set", key, "v"])
    res = run_cli(["memory", "get", key])
    assert json.loads(res.stdout)["value"] == "v"


def test_e2e_memory_set_and_delete_large_key() -> None:
    """Test setting and then deleting a very large key."""
    key = "k" * 2048
    run_cli(["memory", "set", key, "v"])
    run_cli(["memory", "delete", key])
    res = run_cli(["memory", "get", key])
    assert res.returncode != 0 or "null" in res.stdout


def test_e2e_memory_list_with_no_keys_is_empty() -> None:
    """Test that listing an empty store returns an empty list of keys."""
    run_cli(["memory", "clear"])
    res = run_cli(["memory", "list"])
    data = json.loads(res.stdout)
    assert data["keys"] == []


def test_e2e_memory_clear_twice_is_idempotent() -> None:
    """Test that clearing the store twice does not cause an error."""
    run_cli(["memory", "clear"])
    res = run_cli(["memory", "clear"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert "cleared" in data["status"] or "cleared" in data.get("message", "")


def test_e2e_memory_set_then_clear_and_get_returns_none() -> None:
    """Test getting a key after it has been cleared."""
    run_cli(["memory", "set", "will_clear", "gone"])
    run_cli(["memory", "clear"])
    res = run_cli(["memory", "get", "will_clear"])
    assert res.returncode != 0 or "null" in res.stdout or "None" in res.stdout


def test_e2e_memory_get_with_quiet_returns_nothing() -> None:
    """Test that --quiet suppresses output for the get command."""
    run_cli(["memory", "set", "foo", "bar"])
    res = run_cli(["memory", "get", "foo", "--quiet"])
    assert res.stdout.strip() == "" or res.stdout.strip() == "{}"


def test_e2e_memory_list_with_debug_prints_keys() -> None:
    """Test the list command with the --debug flag."""
    run_cli(["memory", "set", "debugkey", "debugval"])
    res = run_cli(["memory", "list", "--debug"])
    assert "debugkey" in res.stdout


def test_e2e_memory_set_and_get_case_sensitive() -> None:
    """Test that keys are case-sensitive."""
    run_cli(["memory", "set", "CaseKey", "UPPER"])
    run_cli(["memory", "set", "casekey", "lower"])
    up = run_cli(["memory", "get", "CaseKey"])
    lo = run_cli(["memory", "get", "casekey"])
    assert json.loads(up.stdout)["value"] == "UPPER"
    assert json.loads(lo.stdout)["value"] == "lower"


def test_e2e_memory_set_empty_key_fails() -> None:
    """Test that setting an empty key fails."""
    res = run_cli(["memory", "set", "", "value"])
    assert res.returncode != 0


def test_e2e_memory_set_none_value_stored_as_empty_string() -> None:
    """Test that a value 'None' is stored as a literal string."""
    res = run_cli(["memory", "set", "nonekey", "None"])
    assert res.returncode == 0
    getres = run_cli(["memory", "get", "nonekey"])
    v = json.loads(getres.stdout)["value"]
    assert v == "None"


def test_e2e_memory_list_format_json_yaml_agree() -> None:
    """Test that JSON and YAML list outputs contain the same keys."""
    run_cli(["memory", "set", "a", "1"])
    run_cli(["memory", "set", "b", "2"])
    res_json = run_cli(["memory", "list", "--format", "json"])
    res_yaml = run_cli(["memory", "list", "--format", "yaml"])
    keys_json = set(json.loads(res_json.stdout)["keys"])
    keys_yaml = set(yaml.safe_load(res_yaml.stdout)["keys"])
    assert keys_json == keys_yaml


def test_e2e_memory_delete_with_quiet_suppresses_output() -> None:
    """Test that --quiet suppresses output for the delete command."""
    run_cli(["memory", "set", "silentdel", "yes"])
    res = run_cli(["memory", "delete", "silentdel", "--quiet"])
    assert res.stdout.strip() == "" or res.stdout.strip() == "{}"


def test_e2e_memory_verbose_root_and_subcommands() -> None:
    """Test that --verbose adds context to root and subcommand outputs."""
    res = run_cli(["memory", "--verbose"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert "python" in data
    assert "platform" in data

    res = run_cli(["memory", "set", "vkey", "vval", "-v"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert "python" in data
    assert "platform" in data

    run_cli(["memory", "set", "gvkey", "gvval"])
    res = run_cli(["memory", "get", "gvkey", "--verbose"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert "python" in data
    assert "platform" in data

    run_cli(["memory", "set", "dvkey", "dvval"])
    res = run_cli(["memory", "delete", "dvkey", "-v"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert "python" in data
    assert "platform" in data

    run_cli(["memory", "set", "lvkey", "lvval"])
    res = run_cli(["memory", "list", "--verbose"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert "python" in data
    assert "platform" in data

    res = run_cli(["memory", "clear", "--verbose"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert "python" in data
    assert "platform" in data


def test_e2e_memory_pretty_and_no_pretty() -> None:
    """Test the --pretty and --no-pretty flags affect output formatting."""
    run_cli(["memory", "clear"])
    for k in ("p1", "p2"):
        run_cli(["memory", "set", k, "v"])
    res = run_cli(["memory", "list", "-f", "json", "--pretty"])
    assert res.returncode == 0
    assert len(res.stdout.splitlines()) > 1
    res = run_cli(["memory", "list", "-f", "json", "--no-pretty"])
    assert res.returncode == 0
    assert len(res.stdout.splitlines()) == 1


def test_e2e_memory_format_case_insensitive_and_last_wins() -> None:
    """Test format flag is case-insensitive and the last one wins."""
    run_cli(["memory", "clear"])
    run_cli(["memory", "set", "fk", "fv"])
    res = run_cli(["memory", "get", "fk", "--format", "YAML"])
    assert res.returncode == 0
    data = yaml.safe_load(res.stdout)
    assert data["value"] == "fv"
    res = run_cli(["memory", "get", "fk", "--format", "yaml", "--format", "json"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert data["value"] == "fv"


@pytest.mark.parametrize("sub", ["", "set foo bar", "get foo", "delete foo", "list"])
def test_e2e_memory_subcommand_help_contract(sub: str) -> None:
    """Test that help output is consistent and well-formed for all subcommands."""
    args = ["memory"] + (sub.split() if sub else []) + ["--help"]
    res = run_cli(args)
    assert res.returncode == 0
    out = res.stdout.lower()
    assert out.lstrip().startswith("usage:")
    for flag in (
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
    ):
        assert flag in out


def test_e2e_memory_unknown_flag_errors() -> None:
    """Test that an unknown flag produces a structured error."""
    res = run_cli(["memory", "set", "ukey", "uval", "--notaflag"])
    assert res.returncode == 2
    payload = json.loads(res.stdout or res.stderr)
    assert "error" in payload
    assert payload.get("code") == 2


def test_e2e_memory_ascii_hygiene(monkeypatch: Any) -> None:
    """Test that non-ASCII environment variables are handled gracefully."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/tmp/\u2603")  # noqa: S108
    res = run_cli(["memory"])
    assert res.returncode == 3
    payload = json.loads(res.stdout or res.stderr)
    assert "error" in payload
    assert payload.get("code") == 3


def test_e2e_memory_boundary_key_validation() -> None:
    """Test that invalid keys (too long, contains spaces/newlines) are rejected."""
    key = "k" * 4097
    res = run_cli(["memory", "set", key, "v"])
    assert res.returncode != 0
    p = json.loads(res.stdout or res.stderr)
    assert p.get("code") == 2

    res = run_cli(["memory", "set", "bad key", "v"])
    assert res.returncode != 0
    p = json.loads(res.stdout or res.stderr)
    assert p.get("code") == 2

    res = run_cli(["memory", "set", "bad\nkey", "v"])
    assert res.returncode != 0
    p = json.loads(res.stdout or res.stderr)
    assert p.get("code") == 2


def test_e2e_memory_sigint_does_not_leak_traceback(tmp_path: Path) -> None:
    """Test that interrupting the process does not leak a stack trace."""
    cmd = [sys.executable, "-m", "bijux_cli", "memory", "list"]
    proc = Popen(cmd, stdout=PIPE, stderr=PIPE, text=True)  # noqa: S603
    time.sleep(0.1)
    proc.send_signal(signal.SIGINT)
    out, err = proc.communicate(timeout=2)
    assert proc.returncode != 0
    msg = (out + err).lower()
    assert "interrupt" in msg or "signal" in msg


def test_e2e_memory_clear_idempotent_and_concurrent() -> None:
    """Test that `clear` is idempotent and safe to run concurrently."""
    run_cli(["memory", "clear"])
    res = run_cli(["memory", "clear"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert "cleared" in data.get("status", "")

    def do_clear() -> None:
        """Executes the 'memory clear' CLI command."""
        run_cli(["memory", "clear"])

    def do_list() -> None:
        """Executes the 'memory list' CLI command."""
        run_cli(["memory", "list"])

    threads = [threading.Thread(target=do_clear) for _ in range(3)] + [
        threading.Thread(target=do_list) for _ in range(3)
    ]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    res = run_cli(["memory", "list"])
    assert res.returncode == 0
    data = json.loads(res.stdout)
    assert isinstance(data.get("keys"), list)
