# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end contract tests for the `bijux config` command."""

from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import resource
import string
from subprocess import PIPE, Popen
import sys
import tempfile
import time
from typing import Any

from hypothesis import assume, given, settings
from hypothesis import strategies as st
import pytest
import yaml  # pyright: ignore[reportMissingModuleSource]

from tests.e2e.conftest import BIN, assert_log_has, assert_text, run_cli

if sys.platform != "win32":
    import fcntl

is_windows = os.name == "nt"


def test_e2e_config_set_get(tmp_path: Path) -> None:
    """Test setting and then getting a configuration value."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "set", "foo=bar"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "status", "updated")
    res = run_cli(["config", "get", "foo"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", "bar")


def test_e2e_config_export(tmp_path: Path) -> None:
    """Test exporting the configuration to a file."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "xyz=1"], env=env)
    out_file = tmp_path / "out.env"
    res = run_cli(["config", "export", str(out_file)], env=env)
    assert res.returncode == 0
    assert out_file.exists()
    assert "BIJUXCLI_XYZ=1\n" in out_file.read_text()


def test_e2e_config_reload(tmp_path: Path) -> None:
    """Test that reloading the config works as expected."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    (tmp_path / ".env").write_text("BIJUXCLI_FOO=bar\n")
    run_cli(["config", "set", "foo=baz"], env=env)
    res = run_cli(["config", "reload"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "status", "reloaded")
    res = run_cli(["config", "get", "foo"], env=env)
    assert_log_has(res.stdout, "value", "baz")


def test_e2e_config_clear(tmp_path: Path) -> None:
    """Test that clearing the configuration removes all values."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "key1=val1"], env=env)
    run_cli(["config", "set", "key2=val2"], env=env)
    res = run_cli(["config", "clear"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "status", "cleared")
    out_file = tmp_path / "out.env"
    run_cli(["config", "export", str(out_file)], env=env)
    assert out_file.read_text() == ""


def test_e2e_config_set_invalid_pair() -> None:
    """Test that setting an invalid key-value pair fails."""
    res = run_cli(["config", "set", "NOEQUALS"])
    assert res.returncode == 2
    assert_text(
        res, "Invalid argument: KEY=VALUE required"
    )  # Update to match new error


def test_e2e_config_get_unknown_key() -> None:
    """Test that getting a non-existent key fails."""
    res = run_cli(["config", "get", "does_not_exist"])
    assert res.returncode == 2
    assert_text(res, "Config key not found")


def test_e2e_config_load_nonexistent(tmp_path: Path) -> None:
    """Test that loading a non-existent file fails."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "load", str(tmp_path / "nonexistent.env")], env=env)
    assert res.returncode == 2
    assert_text(res, "Config file not found")


def test_e2e_config_set_case_sensitivity(tmp_path: Path) -> None:
    """Test that keys are handled case-insensitively."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "KEY=val"], env=env)
    res = run_cli(["config", "get", "key"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", "val")


def test_e2e_config_set_empty_value(tmp_path: Path) -> None:
    """Test setting a key to an empty value."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "key="], env=env)
    res = run_cli(["config", "get", "key"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", "")


def test_e2e_config_export_read_only(tmp_path: Path) -> None:
    """Test that exporting to a read-only location fails gracefully."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "foo=bar"], env=env)
    read_only_dir = tmp_path / "readonly"
    read_only_dir.mkdir()
    read_only_dir.chmod(0o555)
    res = run_cli(["config", "export", str(read_only_dir / "out.env")], env=env)
    assert res.returncode == 2
    assert_text(res, "Permission denied")


def test_e2e_config_malformed_env(tmp_path: Path) -> None:
    """Test that loading a malformed .env file fails."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    (tmp_path / ".env").write_text("MALFORMED")
    res = run_cli(["config", "load", str(tmp_path / ".env")], env=env)
    assert res.returncode == 2
    assert_text(res, "Malformed")


def test_e2e_config_set_numeric_value(tmp_path: Path) -> None:
    """Test that numeric values are correctly decoded when appropriate."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "TIMEOUT=120"], env=env)
    res = run_cli(["config", "get", "TIMEOUT"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", "120")


def test_e2e_config_set_boolean_value(tmp_path: Path) -> None:
    """Test that boolean values are correctly decoded when appropriate."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "ENABLED=true"], env=env)
    res = run_cli(["config", "get", "ENABLED"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", True)


def test_e2e_config_reload_no_file(tmp_path: Path) -> None:
    """Test that reloading fails if no config file has been used."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "reload"], env=env)
    assert res.returncode == 2
    assert_text(res, "Config.reload() called before load()")


def test_e2e_config_set_malformed_key(tmp_path: Path) -> None:
    """Test that setting a key with no name fails."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "set", "=value"], env=env)
    assert res.returncode == 2
    assert_text(res, "Key cannot be empty")


def test_e2e_config_export_invalid_path(tmp_path: Path) -> None:
    """Test that exporting to an invalid file path fails."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "export", "/invalid/path/out.env"], env=env)
    assert res.returncode == 2
    assert_text(res, "No such file or directory")


def test_e2e_config_debug_mode(tmp_path: Path) -> None:
    """Test that the --debug flag is accepted."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "set", "foo=bar", "--debug"], env=env)
    assert res.returncode == 0


def test_e2e_config_yaml_output(tmp_path: Path) -> None:
    """Test that YAML output format works correctly."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "set", "foo=bar", "--format", "yaml"], env=env)
    assert res.returncode == 0
    payload = yaml.safe_load(res.stdout)
    assert payload == {"status": "updated", "key": "foo", "value": "bar"}


def test_e2e_config_set_piped_input(tmp_path: Path) -> None:
    """Test setting a config value via piped stdin."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    proc = Popen(  # noqa: S603
        [str(BIN), "config", "set"],
        env=env,
        stdin=PIPE,
        stdout=PIPE,
        stderr=PIPE,
        text=True,
        start_new_session=True,
    )
    stdout, stderr = proc.communicate(input="foo=bar", timeout=5)
    assert proc.returncode == 0
    assert_text(stdout + stderr, "updated")


def test_e2e_config_export_empty(tmp_path: Path) -> None:
    """Test that exporting an empty config results in an empty file."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    out_file = tmp_path / "out.env"
    res = run_cli(["config", "export", str(out_file)], env=env)
    assert res.returncode == 0
    assert out_file.read_text() == ""


def test_e2e_config_default_no_subcommand(tmp_path: Path) -> None:
    """Test that running 'config' with no subcommand displays the config."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config"], env=env)
    assert res.returncode == 0
    payload = json.loads(res.stdout)
    assert isinstance(payload, dict)


def test_e2e_config_concurrent_config_set(tmp_path: Path) -> None:
    """Test that concurrent 'config set' operations do not corrupt the file."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}

    def run_set(i: int) -> None:
        """Executes the 'config set' CLI command with a unique, timestamped value."""
        val = f"value_{i}_{int(time.time())}"
        run_cli(["config", "set", f"mykey={val}"], env=env, timeout=15)

    with ThreadPoolExecutor(max_workers=5) as executor:
        list(executor.map(run_set, range(5)))

    out_file = tmp_path / "out.env"
    run_cli(["config", "export", str(out_file)], env=env)
    assert out_file.exists()
    content = out_file.read_text()
    assert content.count("BIJUXCLI_MYKEY=") == 1


def test_e2e_config_concurrent_config_get(tmp_path: Path) -> None:
    """Test that concurrent 'config get' operations work correctly."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "foo=bar"], env=env)

    def run_get(_: int) -> None:
        """Executes the 'config get foo' command and asserts the value is 'bar'."""
        res = run_cli(["config", "get", "foo"], env=env, timeout=15)
        assert res.returncode == 0
        assert_log_has(res.stdout, "value", "bar")

    with ThreadPoolExecutor(max_workers=5) as executor:
        list(executor.map(run_get, range(5)))


def test_e2e_config_set_special_characters(tmp_path: Path) -> None:
    """Test setting and getting a value with special characters."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    special_val = "value_with_@#%&*()_+-=|"
    run_cli(["config", "set", f"special={special_val}"], env=env)
    res = run_cli(["config", "get", "special"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", special_val)


def test_e2e_config_overwrite_existing_key(tmp_path: Path) -> None:
    """Test that setting an existing key overwrites its value."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "overwrite=first"], env=env)
    run_cli(["config", "set", "overwrite=second"], env=env)
    res = run_cli(["config", "get", "overwrite"], env=env)
    assert_log_has(res.stdout, "value", "second")


def test_e2e_config_env_var_override(tmp_path: Path) -> None:
    """Test that BIJUXCLI_CONFIG correctly isolates environments."""
    config1 = tmp_path / ".env1"
    config2 = tmp_path / ".env2"
    run_cli(["config", "set", "foo=bar"], env={"BIJUXCLI_CONFIG": str(config1)})
    run_cli(["config", "set", "foo=baz"], env={"BIJUXCLI_CONFIG": str(config2)})
    res1 = run_cli(["config", "get", "foo"], env={"BIJUXCLI_CONFIG": str(config1)})
    res2 = run_cli(["config", "get", "foo"], env={"BIJUXCLI_CONFIG": str(config2)})
    assert_log_has(res1.stdout, "value", "bar")
    assert_log_has(res2.stdout, "value", "baz")


def test_e2e_config_invalid_format(tmp_path: Path) -> None:
    """Test that an invalid output format fails."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "set", "foo=bar", "--format", "invalid"], env=env)
    assert res.returncode == 2
    assert_text(res, "Unsupported format")


@pytest.mark.skipif(
    is_windows, reason="Symlinks/permissions behave differently on Windows"
)
def test_e2e_config_export_symlink(tmp_path: Path) -> None:
    """Exporting to a symlink should succeed; content must end up either in the
    target file (followed) or at the symlink path (replaced)."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    out_file = tmp_path / "target.env"
    link_path = tmp_path / "symlink.env"

    run_cli(["config", "set", "foo=bar"], env=env)
    out_file.write_text("")
    link_path.symlink_to(out_file)

    res = run_cli(["config", "export", str(link_path)], env=env)
    assert res.returncode == 0

    expected = "BIJUXCLI_FOO=bar"
    target_content = out_file.read_text() if out_file.exists() else ""
    link_content = link_path.read_text() if link_path.exists() else ""

    assert (expected in target_content) or (expected in link_content), (
        f"Expected '{expected}' in either target '{out_file}' or link '{link_path}'.\n"
        f"target.exists={out_file.exists()}, link.exists={link_path.exists()}, "
        f"is_link={link_path.is_symlink()}"
    )


def test_e2e_config_set_atomicity(tmp_path: Path) -> None:
    """Test that 'config set' is an atomic operation."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "foo=bar"], env=env)
    config_path = tmp_path / ".env"
    run_cli(["config", "set", "baz=qux"], env=env)
    assert "BIJUXCLI_BAZ=qux" in config_path.read_text()


def test_e2e_config_file_locked(tmp_path: Path) -> None:
    """Test that setting a value fails if the config file is locked."""
    if sys.platform.startswith("win"):
        pytest.skip("File locking not easily portable on Windows")

    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    config_path = tmp_path / ".env"
    config_path.write_text("")
    with config_path.open("r+") as f:
        fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
        res = run_cli(["config", "set", "locked=1"], env=env)
        assert res.returncode != 0
        fcntl.flock(f, fcntl.LOCK_UN)


def test_e2e_config_symlink_loop(tmp_path: Path) -> None:
    """Test that a symlink loop in the config path is handled."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / "loop.env")}
    (tmp_path / "loop.env").symlink_to(tmp_path / "loop.env")
    res = run_cli(["config", "set", "foo=bar"], env=env)
    assert res.returncode != 0


def test_e2e_config_non_ascii_path(tmp_path: Path) -> None:
    """Test using a config file with non-ASCII characters in the path."""
    conf = tmp_path / "conf-汉字.env"
    env = {"BIJUXCLI_CONFIG": str(conf)}
    res = run_cli(["config", "set", "foo=bar"], env=env)
    assert res.returncode == 3
    from json import loads

    err = loads(res.stderr or "{}")
    assert err.get("code") == 3
    assert err.get("failure") == "ascii"


def test_e2e_config_cross_platform_newlines(tmp_path: Path) -> None:
    """Test that CRLF newlines in config files are handled."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    with open(tmp_path / ".env", "wb") as f:
        f.write(b"BIJUXCLI_FOO=bar\r\n")
    run_cli(["config", "reload"], env=env)
    res = run_cli(["config", "get", "FOO"], env=env)
    assert_log_has(res.stdout, "value", "bar")


def test_e2e_config_binary_file(tmp_path: Path) -> None:
    """Binary/non-text files should be reported as malformed and fail."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    (tmp_path / ".env").write_bytes(b"\x00\x01\x02garbage\n")
    res = run_cli(["config", "load", str(tmp_path / ".env")], env=env)
    assert res.returncode == 2
    msg = (res.stdout + res.stderr).lower()
    assert ("malformed" in msg) or ("invalid" in msg), msg


def test_e2e_config_large_key_value(tmp_path: Path) -> None:
    """Test setting and getting a very large key and value."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    key = "K" * 10000
    val = "V" * 10000
    res = run_cli(["config", "set", f"{key}={val}"], env=env)
    assert res.returncode == 0
    res = run_cli(["config", "get", key], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", val)


def test_e2e_config_idempotent_clear(tmp_path: Path) -> None:
    """Test that clearing an already-empty config does not error."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "clear"], env=env)
    res = run_cli(["config", "clear"], env=env)
    assert res.returncode == 0


def test_e2e_config_help_works() -> None:
    """Test that --help works for all config subcommands."""
    for subcmd in [[], ["set"], ["get"], ["export"], ["import"], ["load"], ["clear"]]:
        res = run_cli(["config", *subcmd, "--help"])
        assert res.returncode == 0
        assert "Usage:" in res.stdout


def test_e2e_config_unicode_locale(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the commands work correctly with a non-ASCII locale."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    monkeypatch.setenv("LC_ALL", "zh_CN.UTF-8")
    res = run_cli(["config", "set", "foo=bar"], env=env)
    assert res.returncode == 0


def test_e2e_config_set_multiple_pairs(tmp_path: Path) -> None:
    """Test setting multiple key-value pairs in sequence."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    pairs = ["a=1", "b=2", "c=3"]
    for p in pairs:
        run_cli(["config", "set", p], env=env)
    for k, v in (("a", "1"), ("b", "2"), ("c", "3")):
        res = run_cli(["config", "get", k], env=env)
        assert_log_has(res.stdout, "value", v)


def test_e2e_config_get_after_clear(tmp_path: Path) -> None:
    """Test that getting a key after clearing the config fails."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "foo=bar"], env=env)
    run_cli(["config", "clear"], env=env)
    res = run_cli(["config", "get", "foo"], env=env)
    assert res.returncode != 0


def test_e2e_config_export_then_import_to_another_env(tmp_path: Path) -> None:
    """Test exporting from one environment and loading into another."""
    env1 = {"BIJUXCLI_CONFIG": str(tmp_path / ".env1")}
    env2 = {"BIJUXCLI_CONFIG": str(tmp_path / ".env2")}
    run_cli(["config", "set", "x=1"], env=env1)
    out_file = tmp_path / "out.env"
    run_cli(["config", "export", str(out_file)], env=env1)
    run_cli(["config", "load", str(out_file)], env=env2)
    res = run_cli(["config", "get", "x"], env=env2)
    assert_log_has(res.stdout, "value", "1")


def test_e2e_config_set_invalid_char_in_key(tmp_path: Path) -> None:
    """Test that setting a key with an invalid character fails."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "set", "inval!d=val"], env=env)
    assert res.returncode != 0
    assert "Invalid key" in res.stderr


def test_e2e_config_set_equals_in_value(tmp_path: Path) -> None:
    """Test that a value can contain an equals sign."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "foo=bar=baz"], env=env)
    res = run_cli(["config", "get", "foo"], env=env)
    assert res.returncode == 0
    assert_log_has(res.stdout, "value", "bar=baz")


def test_e2e_config_ignore_comment_lines_on_load(tmp_path: Path) -> None:
    """Test that comment lines in a config file are ignored on load."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    env_file = tmp_path / "withcomments.env"
    env_file.write_text("# comment\nBIJUXCLI_FOO=bar\n# another\nBIJUXCLI_BAZ=qux\n")
    run_cli(["config", "load", str(env_file)], env=env)
    res = run_cli(["config", "get", "FOO"], env=env)
    assert_log_has(res.stdout, "value", "bar")
    res = run_cli(["config", "get", "BAZ"], env=env)
    assert_log_has(res.stdout, "value", "qux")


def test_e2e_config_set_minimum_and_maximum_length(tmp_path: Path) -> None:
    """Test setting values of minimum and maximum reasonable lengths."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "x=a"], env=env)
    long_val = "v" * 4096
    run_cli(["config", "set", f"long={long_val}"], env=env)
    res = run_cli(["config", "get", "long"], env=env)
    assert_log_has(res.stdout, "value", long_val)


def test_e2e_config_export_and_load_yaml(tmp_path: Path) -> None:
    """Test exporting to YAML and that loading from YAML is not supported."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "foo=bar"], env=env)
    yaml_file = tmp_path / "config.yaml"
    run_cli(["config", "export", str(yaml_file), "--out-format", "yaml"], env=env)
    assert yaml.safe_load(yaml_file.read_text()) == {"FOO": "bar"}
    res = run_cli(["config", "load", str(yaml_file)], env=env)
    assert res.returncode != 0


def test_e2e_config_nonexistent_get_returns_error() -> None:
    """Test that getting a non-existent key returns a specific error message."""
    res = run_cli(["config", "get", "thiskeydoesnotexist"])
    assert res.returncode != 0
    assert "Config key not found" in res.stderr


def test_e2e_config_overwrite_and_restore(tmp_path: Path) -> None:
    """Test overwriting a value and then restoring it from a backup."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "restore=one"], env=env)
    out_file = tmp_path / "backup.env"
    run_cli(["config", "export", str(out_file)], env=env)
    run_cli(["config", "set", "restore=two"], env=env)
    run_cli(["config", "load", str(out_file)], env=env)
    res = run_cli(["config", "get", "restore"], env=env)
    assert_log_has(res.stdout, "value", "one")


def test_e2e_config_set_empty_string() -> None:
    """Test that attempting to set an empty string fails."""
    res = run_cli(["config", "set", ""])
    assert res.returncode != 0
    assert "Invalid" in res.stdout or "Invalid" in res.stderr


def test_e2e_config_case_preservation(tmp_path: Path) -> None:
    """Test that key casing is preserved."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "FooBar=MixedCase"], env=env)
    res = run_cli(["config", "get", "FooBar"], env=env)
    assert_log_has(res.stdout, "value", "MixedCase")


def test_e2e_config_export_to_stdout(tmp_path: Path) -> None:
    """Test exporting the configuration to stdout."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    run_cli(["config", "set", "foo=bar"], env=env)
    res = run_cli(["config", "export", "-"], env=env)
    assert res.returncode == 0
    assert res.stdout == "BIJUXCLI_FOO=bar\n"


def test_e2e_config_set_key_with_space_should_fail(tmp_path: Path) -> None:
    """Test that setting a key containing a space fails."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    res = run_cli(["config", "set", "foo bar=val"], env=env)
    assert res.returncode != 0
    assert "Invalid key" in res.stderr


def test_e2e_config_import_empty_file(tmp_path: Path) -> None:
    """Test that loading an empty file results in an empty configuration."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    empty_file = tmp_path / "empty.env"
    empty_file.write_text("")
    res = run_cli(["config", "load", str(empty_file)], env=env)
    assert res.returncode == 0
    exp = run_cli(["config", "export", "-"], env=env)
    assert exp.stdout.strip() == ""


def test_e2e_config_export_truncates_file(tmp_path: Path) -> None:
    """Test that exporting to a file truncates it before writing."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    out_file = tmp_path / "truncate.env"
    out_file.write_text("garbage")
    run_cli(["config", "set", "foo=bar"], env=env)
    run_cli(["config", "export", str(out_file)], env=env)
    assert "BIJUXCLI_FOO=bar" in out_file.read_text()
    assert "garbage" not in out_file.read_text()


allowed_value_alphabet = "".join(
    c for c in string.printable if c not in "\r\n\t\x0b\x0c"
)


@settings(deadline=None, max_examples=10)
@given(
    key=st.text(
        alphabet=string.ascii_letters + string.digits + "_", min_size=1, max_size=10
    ),
    value=st.text(alphabet=allowed_value_alphabet, min_size=0, max_size=10),
)
def test_e2e_config_fuzz_key_value(key: str, value: str) -> None:
    """Fuzz `config set/get` with valid keys and accepted values."""
    assume("=" not in key and not any(c.isspace() for c in key))

    with tempfile.TemporaryDirectory() as d:
        tmp_path = Path(d)
        env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}

        rc = run_cli(["config", "set", f"{key}={value}"], env=env).returncode
        assert rc == 0, "config set failed"

        out = run_cli(["config", "get", key], env=env)
        assert out.returncode == 0, "config get failed"

        parsed: dict[str, Any] | None = None
        for candidate in (out.stdout, out.stderr):
            try:
                parsed = json.loads(candidate)
                break
            except json.JSONDecodeError:
                continue

        assert parsed is not None, (
            f"No JSON in stdout/stderr: stdout={out.stdout!r} stderr={out.stderr!r}"
        )
        assert "value" in parsed, f"No 'value' key in parsed JSON: {parsed!r}"

        actual = parsed["value"]
        assert actual == value or str(actual) == str(value), (
            f"Mismatch: {actual!r} vs {value!r}"
        )


def test_e2e_config_simulate_crash_during_write(tmp_path: Path) -> None:
    """Simulate a process crash after temp file write but before replacement."""
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    config_path = tmp_path / ".env"
    tmp_file = tmp_path / ".env.tmp"
    run_cli(["config", "set", "foo=bar"], env=env)
    tmp_file.write_text("BIJUXCLI_CRASH=1\n")
    assert config_path.exists()
    run_cli(["config", "set", "baz=qux"], env=env)
    res = run_cli(["config", "get", "baz"], env=env)
    assert_log_has(res.stdout, "value", "qux")


@pytest.mark.skipif(sys.platform == "win32", reason="requires the 'resource' module")
def test_e2e_config_file_descriptor_exhaustion(tmp_path: Path) -> None:
    """Test graceful failure when file descriptors are exhausted."""
    soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
    resource.setrlimit(resource.RLIMIT_NOFILE, (16, hard))
    env = {"BIJUXCLI_CONFIG": str(tmp_path / ".env")}
    try:
        with pytest.raises((OSError, ValueError)):
            run_cli(["config", "set", "foo=bar"], env=env)
    finally:
        resource.setrlimit(resource.RLIMIT_NOFILE, (soft, hard))
