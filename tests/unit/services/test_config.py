# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services config module."""

# pyright: reportPrivateUsage=false

from __future__ import annotations

from collections.abc import Callable
import json
import os
from pathlib import Path
from typing import Any, cast
from unittest.mock import MagicMock, Mock, patch

import pytest

from bijux_cli.contracts import ObservabilityProtocol
from bijux_cli.core.exceptions import CommandError
from bijux_cli.services.config import Config, _detect_symlink_loop, _escape, _unescape


@pytest.fixture
def mock_di() -> Any:
    """Provide a mock dependency injector."""
    di = Mock()
    di.resolve.return_value = Mock(spec=ObservabilityProtocol)
    return di


@pytest.fixture
def mock_log(mock_di: Any) -> ObservabilityProtocol:
    """Provide a mock log service from the DI container."""
    return cast(ObservabilityProtocol, mock_di.resolve.return_value)


@pytest.fixture
def config(mock_di: Any) -> Config:
    """Provide a Config instance with a mocked DI container."""
    return Config(mock_di)


@pytest.fixture
def temp_env_file(tmp_path: Path) -> Path:
    """Provide a temporary .env file path."""
    return tmp_path / ".env"


@pytest.fixture
def cfg(tmp_path: Path) -> Config:
    """Provide a Config instance pointed at a temporary directory."""
    c = Config(MagicMock())
    c._path = tmp_path / ".env"
    c._path.parent.mkdir(parents=True, exist_ok=True)
    return c


def test_escape_basic() -> None:
    """Test escaping of basic strings."""
    assert _escape("hello") == "hello"
    assert _escape("") == ""


def test_escape_special() -> None:
    """Test escaping of special characters."""
    assert _escape('a\\b\nc"') == 'a\\\\b\\nc\\"'


def test_unescape_basic() -> None:
    """Test un-escaping of basic strings."""
    assert _unescape("hello") == "hello"


def test_unescape_special() -> None:
    """Test un-escaping of special characters."""
    assert _unescape('a\\\\b\\nc\\"') == 'a\\b\nc"'


def test_unescape_invalid() -> None:
    """Test that un-escaping an invalid sequence raises a ValueError."""
    with pytest.raises(ValueError, match="Invalid escaped string"):
        _unescape("\\x")


def test_init_load_success(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the constructor successfully loads a valid config file."""
    temp_env_file.write_text("BIJUXCLI_KEY=value\n")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    c = Config(config._di)
    assert c.get("key") == "value"


def test_init_load_missing(config: Config, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the constructor handles a missing config file gracefully."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/nonexistent.env")
    c = Config(config._di)
    assert not c.all()


def test_init_load_error(
    config: Config,
    temp_env_file: Path,
    monkeypatch: pytest.MonkeyPatch,
    mock_log: ObservabilityProtocol,
) -> None:
    """Test that an error during config loading is logged."""
    temp_env_file.write_text("invalid=line\n")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    Config(config._di)
    (cast(Any, mock_log.log)).assert_called()


def test_load_default(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test loading from the default config path."""
    temp_env_file.write_text("KEY=value\n")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config.load()
    assert config.get("key") == "value"


def test_load_explicit(config: Config, temp_env_file: Path) -> None:
    """Test loading from an explicitly provided path."""
    temp_env_file.write_text("EXPLICIT=val\n")
    config.load(temp_env_file)
    assert config.get("explicit") == "val"


def test_load_missing(config: Config) -> None:
    """Test that loading a missing file raises FileNotFoundError."""
    with pytest.raises(FileNotFoundError):
        config.load("/nonexistent")


def test_load_symlink_loop(config: Config, tmp_path: Path) -> None:
    """Test that a symlink loop in the config path is detected."""
    link1 = tmp_path / "link1"
    link2 = tmp_path / "link2"
    link1.symlink_to(link2)
    link2.symlink_to(link1)
    with pytest.raises(CommandError, match="Symlink loop"):
        config.load(link1)


def test_load_malformed_line(config: Config, temp_env_file: Path) -> None:
    """Test that a malformed line in the config file raises an error."""
    temp_env_file.write_text("no_equal\n")
    with pytest.raises(CommandError, match="Malformed line"):
        config.load(temp_env_file)


def test_load_non_ascii(config: Config, temp_env_file: Path) -> None:
    """Test that a non-ASCII value in the config file raises an error."""
    temp_env_file.write_text("KEY=unicodé\n")
    with pytest.raises(CommandError, match="Non-ASCII"):
        config.load(temp_env_file)


def test_load_binary(config: Config, temp_env_file: Path) -> None:
    """Test that attempting to load a binary file raises an error."""
    temp_env_file.write_bytes(b"\x80binary")
    with pytest.raises(CommandError, match="Binary or non-text"):
        config.load(temp_env_file)


def test_load_other_error(config: Config, temp_env_file: Path) -> None:
    """Test that a generic OSError during load is wrapped in a CommandError."""
    temp_env_file.touch()
    with (
        patch.object(Path, "read_text", side_effect=OSError("io")),
        pytest.raises(CommandError),
    ):
        config.load(temp_env_file)


def test_load_prefix_remove(config: Config, temp_env_file: Path) -> None:
    """Test that the 'BIJUXCLI_' prefix is removed from keys during load."""
    temp_env_file.write_text("BIJUXCLI_UPPER=val\n")
    config.load(temp_env_file)
    assert config.get("upper") == "val"


def test_load_lower_key(config: Config, temp_env_file: Path) -> None:
    """Test that keys are lowercased during load."""
    temp_env_file.write_text("UPPER=val\n")
    config.load(temp_env_file)
    assert config.get("upper") == "val"


def test_load_skip_comments(config: Config, temp_env_file: Path) -> None:
    """Test that comment lines are skipped during load."""
    temp_env_file.write_text("# comment\nKEY=val\n")
    config.load(temp_env_file)
    assert config.get("key") == "val"


def test_load_empty_lines(config: Config, temp_env_file: Path) -> None:
    """Test that empty lines are skipped during load."""
    temp_env_file.write_text("\n\nKEY=val\n\n")
    config.load(temp_env_file)
    assert config.get("key") == "val"


def test_load_unescape(config: Config, temp_env_file: Path) -> None:
    """Test that values are correctly unescaped during load."""
    temp_env_file.write_text('KEY="a\\nb"\n')
    config.load(temp_env_file)
    assert config.get("key") == "a\nb"


def test_load_imports_data_without_changing_config_path(
    config: Config, tmp_path: Path
) -> None:
    """Test that `config.load` imports data without altering the primary config path."""
    original_path = config._path
    import_path = tmp_path / "import.env"
    import_path.write_text("NEW_KEY_FROM_LOAD=new_value\n")
    config.load(import_path)
    assert config.get("new_key_from_load") == "new_value"
    assert config._path == original_path


def test_set_many(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that set_many correctly persists multiple values."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config.set_many({"key": "val"})
    assert temp_env_file.read_text() == "BIJUXCLI_KEY=val\n"


def test_set_many_escape(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that set_many correctly escapes special characters in values."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config.set_many({"key": 'a\nb"c'})
    assert temp_env_file.read_text() == 'BIJUXCLI_KEY=a\\nb\\"c\n'


def test_set_many_locked_retry(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that set_many retries when the config file is locked."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    temp_env_file.write_text("")
    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        pytest.raises(CommandError, match="locked after retries"),
    ):
        config.set_many({"key": "val"})


def test_set_many_error_cleanup(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that temporary files are cleaned up after an error in set_many."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    with (
        patch("builtins.open", side_effect=Exception("open fail")),
        pytest.raises(CommandError),
    ):
        config.set_many({"key": "val"})
    assert not temp_env_file.with_suffix(".tmp").exists()


def test_all(config: Config) -> None:
    """Test that all() returns the complete in-memory configuration dictionary."""
    config._data = {"key": "val"}
    assert config.all() == {"key": "val"}


def test_list_keys(config: Config) -> None:
    """Test that list_keys() returns a list of all configuration keys."""
    config._data = cast(dict[str, str], {"a": 1, "b": 2})
    assert sorted(config.list_keys()) == ["a", "b"]


def test_clear(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that clear() removes all keys and deletes the config file."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    temp_env_file.write_text("KEY=val\n")
    config.load()
    config.clear()
    assert not temp_env_file.exists()
    assert not config.all()


def test_clear_no_file(config: Config) -> None:
    """Test that clear() works correctly when no config file exists."""
    config.clear()
    assert not config.all()


def test_clear_locked(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that clear() raises an error if the config file is locked."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    temp_env_file.write_text("")
    config.load()
    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        pytest.raises(CommandError, match="locked"),
    ):
        config.clear()


def test_clear_error(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that a generic error during clear() is handled."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    temp_env_file.write_text("")
    config.load()
    with (
        patch.object(Path, "unlink", side_effect=OSError("unlink fail")),
        pytest.raises(CommandError),
    ):
        config.clear()


def test_get_found(config: Config) -> None:
    """Test getting a value that exists in the config."""
    config._data = {"key": "val"}
    assert config.get("key") == "val"


def test_get_default(config: Config) -> None:
    """Test getting a non-existent key with a default value."""
    assert config.get("missing", "def") == "def"


def test_get_not_found(config: Config, mock_log: ObservabilityProtocol) -> None:
    """Test that getting a non-existent key without a default raises an error."""
    with pytest.raises(CommandError, match="not found"):
        config.get("missing")
    (cast(Any, mock_log.log)).assert_called()


def test_get_env_override(config: Config, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an environment variable overrides a value from the config file."""
    monkeypatch.setenv("BIJUXCLI_KEY", "env")
    config._data = {"key": "file"}
    assert config.get("key") == "env"


def test_get_bool_true(config: Config) -> None:
    """Test that the string 'true' is correctly parsed as a boolean."""
    config._data = {"bool": "true"}
    assert config.get("bool") is True


def test_get_bool_false(config: Config) -> None:
    """Test that the string 'false' is correctly parsed as a boolean."""
    config._data = {"bool": "false"}
    assert config.get("bool") is False


def test_get_prefix_insensitive(config: Config) -> None:
    """Test that the 'BIJUXCLI_' prefix is ignored when getting a key."""
    config._data = {"key": "val"}
    assert config.get("BIJUXCLI_KEY") == "val"


def test_set(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that set() correctly persists a key-value pair."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config.set("key", "val")
    assert config.get("key") == "val"
    assert "BIJUXCLI_KEY=val" in temp_env_file.read_text()


def test_set_existing(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that set() correctly overrides an existing key."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config.set("key", "old")
    config.set("key", "new")
    assert config.get("key") == "new"


def test_set_locked(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that set() raises an error if the config file is locked."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    temp_env_file.write_text("")
    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        pytest.raises(CommandError, match="locked after retries"),
    ):
        config.set("key", "val")


def test_set_error_cleanup(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that temporary files are cleaned up after an error in set()."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    with (
        patch("builtins.open", side_effect=Exception("open fail")),
        pytest.raises(CommandError),
    ):
        config.set("key", "val")
    assert not temp_env_file.with_suffix(".tmp").exists()


def test_reload(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that reload() correctly re-reads the configuration from the file."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    temp_env_file.write_text("KEY=val\n")
    config.load(temp_env_file)
    temp_env_file.write_text("KEY=new\n")
    config.reload()
    assert config.get("key") == "new"


def test_reload_no_path(config: Config) -> None:
    """Test that reload() raises an error if no config path is set."""
    config._path = None
    with pytest.raises(CommandError):
        config.reload()


def test_reload_missing(config: Config, temp_env_file: Path) -> None:
    """Test that reload() raises an error if the config file is missing."""
    temp_env_file.touch()
    config._path = temp_env_file
    temp_env_file.unlink()
    with pytest.raises(CommandError):
        config.reload()


def test_export_env(config: Config, tmp_path: Path) -> None:
    """Test exporting the configuration to a .env file format."""
    config._data = {"key": "val"}
    dest = tmp_path / "export.env"
    config.export(dest)
    assert dest.read_text() == "BIJUXCLI_KEY=val\n"


def test_export_json(config: Config, tmp_path: Path) -> None:
    """Test exporting the configuration to JSON format."""
    config._data = {"key": "val"}
    dest = tmp_path / "export.json"
    config.export(dest)
    assert json.loads(dest.read_text()) == {"KEY": "val"}


def test_export_yaml(config: Config, tmp_path: Path) -> None:
    """Test exporting the configuration to YAML format."""
    config._data = {"key": "val"}
    dest = tmp_path / "export.yaml"
    config.export(dest)
    assert "KEY: val" in dest.read_text()


def test_export_stdout(config: Config, capsys: pytest.CaptureFixture[str]) -> None:
    """Test exporting the configuration to stdout."""
    config._data = {"key": "val"}
    config.export("-")
    captured = capsys.readouterr()
    assert "BIJUXCLI_KEY=val" in captured.out


def test_export_invalid_fmt(config: Config) -> None:
    """Test that exporting to an invalid format raises an error."""
    with pytest.raises(CommandError):
        config.export("file", "invalid")


def test_export_no_yaml(config: Config, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that exporting to YAML fails if PyYAML is not installed."""
    monkeypatch.setattr("bijux_cli.services.config.yaml", None)
    with pytest.raises(CommandError, match="PyYAML"):
        config.export("file.yaml")


def test_export_no_dir(config: Config) -> None:
    """Test that exporting to a non-existent directory raises an error."""
    with pytest.raises(CommandError, match="No such"):
        config.export("/nonexistent/dir/file")


def test_export_permission(config: Config) -> None:
    """Test that a permission error during export is handled."""
    with (
        patch.object(Path, "exists", return_value=True),
        patch.object(os, "access", return_value=False),
        pytest.raises(CommandError, match="Permission denied"),
    ):
        config.export("/no/perm/file")


def test_export_locked(config: Config, tmp_path: Path) -> None:
    """Test that exporting to a locked file raises an error."""
    dest = tmp_path / "locked"
    dest.write_text("")
    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        pytest.raises(CommandError, match="locked"),
    ):
        config.export(dest)


def test_delete(config: Config) -> None:
    """Test that delete() removes a key from the in-memory config."""
    config._data = {"key": "val"}
    config.delete("key")
    assert "key" not in config._data


def test_delete_not_found(config: Config, mock_log: ObservabilityProtocol) -> None:
    """Test that deleting a non-existent key raises an error."""
    with pytest.raises(CommandError, match="not found"):
        config.delete("missing")
    (cast(Any, mock_log.log)).assert_called()


def test_unset_alias(config: Config) -> None:
    """Test that unset() is a successful alias for delete()."""
    config._data = {"key": "val"}
    config.unset("key")
    assert "key" not in config._data


def test_save(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that save() persists the in-memory configuration."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config._data = {"key": "val"}
    config.save()
    assert "BIJUXCLI_KEY=val" in temp_env_file.read_text()


def test_save_no_path(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that save() determines and sets the default path if not already set."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config._data = {"key": "val"}
    config.save()
    assert config._path == temp_env_file


def test_save_error(config: Config, mock_log: ObservabilityProtocol) -> None:
    """Test that an error during save() is handled."""
    with patch.object(  # noqa: SIM117
        Config, "set_many", side_effect=Exception("save fail")
    ):
        with pytest.raises(CommandError):
            config.save()
    (cast(Any, mock_log.log)).assert_called()


def test_validate_config_path_device() -> None:
    """Test that validating a path pointing to a device file raises an error."""
    with pytest.raises(CommandError, match="device file"):
        Config._validate_config_path(Path("/dev/null"))


def test_validate_config_path_windows() -> None:
    """Test that validating a path pointing to a Windows device file raises an error."""
    with pytest.raises(CommandError, match="device file"):
        Config._validate_config_path(Path("\\\\.\\nul"))


def test_preflight_symlink_loop(config: Config, tmp_path: Path) -> None:
    """Test that the preflight write check detects symlink loops."""
    link1 = tmp_path / "link1"
    link2 = tmp_path / "link2"
    link1.symlink_to(link2)
    link2.symlink_to(link1)
    with pytest.raises(CommandError, match="Symlink loop"):
        Config._preflight_write(link1)


def test_preflight_locked(config: Config, temp_env_file: Path) -> None:
    """Test that the preflight write check detects a locked file."""
    temp_env_file.write_text("")
    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        pytest.raises(CommandError, match="locked"),
    ):
        Config._preflight_write(temp_env_file)


def test_preflight_no_file(config: Config, temp_env_file: Path) -> None:
    """Test that the preflight write check passes for a non-existent file."""
    temp_env_file.unlink(missing_ok=True)
    Config._preflight_write(temp_env_file)


def test_set_many_no_path(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that set_many() determines and sets the default path if not already set."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config.set_many({"key": "val"})
    assert config._path == temp_env_file


def test_get_env_no_prefix(config: Config, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test getting an environment variable without the 'BIJUXCLI_' prefix."""
    monkeypatch.setenv("BIJUXCLI_KEY", "env")
    assert config.get("key") == "env"


def test_get_not_str_no_bool(config: Config) -> None:
    """Test that a non-boolean string value is returned as a string."""
    config._data = {"num": "123"}
    assert config.get("num") == "123"


def test_export_auto_fmt(config: Config, tmp_path: Path) -> None:
    """Test that export() automatically determines format from file extension."""
    config._data = {"key": "val"}
    dest_yaml = tmp_path / "conf.yaml"
    config.export(dest_yaml)
    assert "KEY: val" in dest_yaml.read_text()


def test_export_stdout_json(config: Config, capsys: pytest.CaptureFixture[str]) -> None:
    """Test exporting to stdout in JSON format."""
    config._data = {"key": "val"}
    config.export("-", "json")
    captured = capsys.readouterr()
    assert json.loads(captured.out) == {"KEY": "val"}


def test_export_locked_retry(config: Config, tmp_path: Path) -> None:
    """Test that export() retries when the destination file is locked."""
    dest = tmp_path / "locked"
    dest.write_text("")
    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        pytest.raises(CommandError, match="locked"),
    ):
        config.export(dest)


def test_export_error_exists(config: Config) -> None:
    """Test that exporting to a non-existent directory path fails."""
    with pytest.raises(CommandError, match="No such"):
        config.export("/nonexistent/file")


def test_export_permission_dir(config: Config) -> None:
    """Test that exporting to a directory without write permission fails."""
    with (
        patch.object(os, "access", return_value=False),
        patch.object(Path, "exists", return_value=True),
        pytest.raises(CommandError, match="Permission denied"),
    ):
        config.export("/no/perm/dir/file")


def test_export_permission_file(config: Config, tmp_path: Path) -> None:
    """Test that exporting to a read-only file fails."""
    dest = tmp_path / "no_perm"
    dest.touch()
    dest.chmod(0o444)
    with pytest.raises(CommandError, match="Permission denied"):
        config.export(dest)


def test_delete_persist(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that delete() operation is correctly persisted."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config.set("key", "val")
    config.delete("key")
    assert "KEY" not in temp_env_file.read_text()


def test_delete_locked(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that delete() raises an error if the config file is locked."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    temp_env_file.write_text("KEY=val\n")
    config.load()
    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        pytest.raises(CommandError, match="locked after retries"),
    ):
        config.delete("key")


def test_save_empty(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that save() writes an empty file if the config is empty."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config.save()
    assert temp_env_file.read_text() == ""


def test_preflight_depth_exceed(config: Config, tmp_path: Path) -> None:
    """Test that the preflight write check detects excessive symlink depth."""
    curr = tmp_path / "start"
    for i in range(20):
        next_link = tmp_path / f"link{i}"
        curr.symlink_to(next_link)
        curr = next_link
    with pytest.raises(CommandError, match="depth exceeded"):
        Config._preflight_write(tmp_path / "start")


def test_set_many_validate(config: Config, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that set_many() validates the config path before writing."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/dev/null")
    config._path = None
    with pytest.raises(CommandError):
        config.set_many({})


def test_set_validate(config: Config, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that set() validates the config path before writing."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/dev/null")
    config._path = None
    with pytest.raises(CommandError):
        config.set("key", "val")


def test_save_validate(config: Config, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that save() validates the config path before writing."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "/dev/null")
    config._path = None
    with pytest.raises(CommandError):
        config.save()


def test_export_auto_env(config: Config, tmp_path: Path) -> None:
    """Test auto-detection of .env format for export."""
    config.clear()
    dest = tmp_path / "conf.env"
    config.export(dest)
    assert not dest.read_text()


def test_export_auto_json(config: Config, tmp_path: Path) -> None:
    """Test auto-detection of .json format for export."""
    config.clear()
    dest = tmp_path / "conf.json"
    config.export(dest)
    assert dest.read_text() == "{}\n"


def test_export_stdout_empty(
    config: Config, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test exporting an empty config to stdout."""
    config.clear()
    config.export("-")
    captured = capsys.readouterr()
    assert not captured.out


def test_save_empty_file(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that saving an empty config results in an empty file."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    config.save()
    assert not temp_env_file.read_text()


def test_symlink_relative_path_branch(tmp_path: Path) -> None:
    """Test symlink loop detection with a relative target path."""
    file = tmp_path / "target"
    file.write_text("foo")
    rel_link = tmp_path / "link"
    rel_link.symlink_to("target")
    _detect_symlink_loop(rel_link)


def test_config_init_command_error_logs(
    mock_di: Any,
    mock_log: ObservabilityProtocol,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a CommandError during initialization is logged."""

    class DummyConfig(Config):
        def load(self, path: str | Path | None = None) -> None:
            raise CommandError("fail")

    dummy_di = Mock()
    dummy_di.resolve.return_value = mock_log
    DummyConfig(dummy_di)
    cast(Mock, mock_log.log).assert_called_with(
        "error", "Auto-load of config failed during init: fail", extra={}
    )


def test_set_many_error_cleanup_exists(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that an existing temporary file is cleaned up after an error in set_many."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    tmp_file = temp_env_file.with_suffix(".tmp")
    tmp_file.touch()
    with (
        patch("builtins.open", side_effect=Exception("fail")),
        pytest.raises(CommandError),
    ):
        config.set_many({"key": "val"})
    assert not tmp_file.exists()


def test_set_many_locked_retry_cleanup(
    config: Config, temp_env_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the temporary file is cleaned up after retry exhaustion in set_many."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    tmp_file = temp_env_file.with_suffix(".tmp")
    tmp_file.touch()
    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        pytest.raises(CommandError, match="locked after retries"),
    ):
        config.set_many({"key": "val"})
    assert not tmp_file.exists()


def test_preflight_write_unlock_path(config: Config, temp_env_file: Path) -> None:
    """Test that the preflight write check correctly unlocks the file."""
    temp_env_file.write_text("foo")
    Config._preflight_write(temp_env_file)


def test_config_init_handles_file_not_found(mock_di: Any) -> None:
    """Test that a FileNotFoundError during initialization is handled."""

    class DummyConfig(Config):
        def load(self, path: str | Path | None = None) -> None:
            raise FileNotFoundError("missing")

    dummy = DummyConfig(mock_di)
    cast(Mock, dummy._log.log).assert_not_called()


def test_export_parent_dir_permission_denied(config: Config, tmp_path: Path) -> None:
    """Test that exporting to a directory with no write permissions fails."""
    config._data = {"key": "val"}
    dest = tmp_path / "conf.env"

    with (
        patch.object(
            Path, "exists", autospec=True, side_effect=lambda p: p == dest.parent
        ),
        patch("os.access", side_effect=lambda p, m: Path(p) != dest.parent),
        pytest.raises(CommandError, match="Permission denied"),
    ):
        config.export(dest)


def test_detect_symlink_loop_os_error(tmp_path: Path, mock_di: Any) -> None:
    """Test that an OSError during symlink detection is handled."""
    target = tmp_path / "foo"
    link = tmp_path / "bar"
    link.symlink_to(target)
    with (
        patch("os.readlink", side_effect=OSError("boom")),
        pytest.raises(CommandError, match="Symlink loop detected"),
    ):
        _detect_symlink_loop(link)


def test_delete_inner_write_error_cleans_tmp(
    tmp_path: Path, mock_di: Any, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that an error during the inner write of delete cleans up the temp file."""
    env = tmp_path / ".env"
    env.write_text("KEY=val\n")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(env))
    cfg = Config(mock_di)
    cfg.load()
    tmpfile = env.with_suffix(".tmp")
    tmpfile.write_text("")

    with (
        patch("builtins.open", side_effect=Exception("oops")),
        pytest.raises(CommandError, match="oops"),
    ):
        cfg.delete("key")
    assert not tmpfile.exists()


def test_set_replace_failure_cleans_tmp(
    tmp_path: Path, mock_di: Any, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that a failure to replace the config file during set cleans up the temp file."""
    env = tmp_path / ".env"
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(env))
    monkeypatch.setattr(
        "bijux_cli.services.config._detect_symlink_loop",
        lambda path: None,
    )

    cfg = Config(mock_di)
    cfg._path = None
    tmpfile = env.with_suffix(".tmp")

    with (
        patch("fcntl.flock", lambda fd, flags: None),
        patch("pathlib.Path.replace", side_effect=ValueError("replace fail")),
        pytest.raises(CommandError, match="replace fail"),
    ):
        cfg.set("key", "value")

    assert not tmpfile.exists()


def test_delete_replace_failure_cleans_tmp(
    tmp_path: Path, mock_di: Any, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that a failure to replace the config file during delete cleans up the temp file."""
    env = tmp_path / ".env"
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(env))
    env.write_text("A=1\nB=2\n")
    monkeypatch.setattr(
        "bijux_cli.services.config._detect_symlink_loop",
        lambda path: None,
    )

    cfg = Config(mock_di)
    cfg.load()
    tmpfile = env.with_suffix(".tmp")

    with (
        patch("fcntl.flock", lambda fd, flags: None),
        patch("pathlib.Path.replace", side_effect=ValueError("replace fail")),
        pytest.raises(CommandError, match="replace fail"),
    ):
        cfg.delete("a")

    assert not tmpfile.exists()


def test_delete_writes_remaining_items(
    tmp_path: Path, mock_di: Any, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that delete correctly writes the remaining items to the config file."""
    env = tmp_path / ".env"
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(env))
    env.write_text("A=1\nB=2\nC=3\n")

    cfg = Config(mock_di)
    cfg.load()

    cfg.delete("b")

    lines = set(env.read_text().splitlines())
    assert lines == {"BIJUXCLI_A=1", "BIJUXCLI_C=3"}


@pytest.mark.parametrize("method", ["set_many", "set", "delete"])
def test_tmp_cleanup_branch(
    config: Config,
    temp_env_file: Path,
    monkeypatch: pytest.MonkeyPatch,
    method: str,
) -> None:
    """Test the specific cleanup branch for temporary files after retry exhaustion."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(temp_env_file))
    config._path = None
    tmp_file = temp_env_file.with_suffix(".tmp")
    tmp_file.touch()

    action = None  # pyright: ignore[reportAssignmentType]
    if method == "set_many":

        def action() -> None:
            """Define the set_many action."""
            config.set_many({"key": "val"})

    elif method == "set":

        def action() -> None:
            """Define the set action."""
            config.set("key", "val")

    elif method == "delete":
        config._data = {"key": "val"}

        def action() -> None:
            """Define the delete action."""
            config.delete("key")

    assert action is not None

    with patch("fcntl.flock", side_effect=BlockingIOError):
        orig_exists = tmp_file.exists

        def fake_exists(self: Path) -> bool:
            """Simulate the temporary file always existing for the cleanup check."""
            if self == tmp_file:
                return True
            return orig_exists()

        with (
            patch("pathlib.Path.exists", fake_exists),
            patch("pathlib.Path.unlink") as mock_unlink,
        ):
            with pytest.raises(CommandError, match="locked after retries"):
                action()
            mock_unlink.assert_called_once()


@pytest.mark.parametrize("method", ["set_many", "set", "delete"])
def test_config_retry_exhaustion_cleans_tmp(tmp_path: Path, method: str) -> None:
    config = Config(MagicMock())
    config._path = tmp_path / ".env"
    config._data = {"foo": "bar"}

    tmp_file = config._path.with_suffix(".tmp")
    tmp_file.touch()

    with (
        patch("builtins.open", side_effect=BlockingIOError),
        patch.object(Path, "exists", return_value=True),
        patch.object(Path, "unlink", wraps=tmp_file.unlink) as mock_unlink,
    ):

        def _set_many() -> None:
            config.set_many({"foo": "bar"})

        def _set() -> None:
            config.set("foo", "baz")

        def _delete() -> None:
            config.delete("foo")

        method_map: dict[str, Callable[[], None]] = {
            "set_many": _set_many,
            "set": _set,
            "delete": _delete,
        }

        with pytest.raises(CommandError) as exc_info:
            method_map[method]()  # typed callable

        assert mock_unlink.called
        assert "File locked after retries" in str(exc_info.value)


@pytest.mark.parametrize(
    ("method", "args"),
    [("set_many", ({"foo": "bar"},)), ("set", ("foo", "baz")), ("delete", ("foo",))],
)
def test_config_cleanup_tmp_on_retry_exhaustion(
    tmp_path: Path, method: str, args: tuple[Any, ...]
) -> None:
    """Test temporary file cleanup on retry exhaustion for various methods."""
    config = Config(MagicMock())
    config._path = tmp_path / ".env"
    config._data = {"foo": "bar"}

    tmp_file = config._path.with_suffix(".tmp")
    tmp_file.touch()

    with (
        patch("builtins.open", side_effect=BlockingIOError),
        patch.object(Path, "exists", return_value=True),
        patch.object(Path, "unlink") as mock_unlink,
    ):
        method_func = getattr(config, method)

        with pytest.raises(CommandError) as exc_info:
            method_func(*args)

        mock_unlink.assert_called_once_with()
        assert "File locked after retries" in str(exc_info.value)


def _cfg(tmp_path: Path) -> Config:
    """Create a Config instance for testing."""
    cfg = Config(MagicMock())
    cfg._path = tmp_path / ".env"
    cfg._data = {"foo": "bar"}
    return cfg


def test_set_many_exhausts_retries_and_unlinks_tmp(tmp_path: Path) -> None:
    """Test that set_many cleans up the temp file after exhausting lock retries."""
    cfg = _cfg(tmp_path)

    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("time.sleep", return_value=None),
    ):
        assert cfg._path is not None
        tmp_file = cfg._path.with_suffix(".tmp")
        tmp_file.touch()

        with pytest.raises(CommandError) as exc:
            cfg.set_many({"foo": "baz"})

    assert "File locked after retries" in str(exc.value)
    assert not tmp_file.exists()


def test_set_retry_exhaustion_unlinks_tmp(tmp_path: Path) -> None:
    """Test that set cleans up the temp file after exhausting lock retries."""
    cfg = _cfg(tmp_path)

    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("time.sleep", return_value=None),
    ):
        assert cfg._path is not None
        tmp_file = cfg._path.with_suffix(".tmp")
        tmp_file.touch()

        with pytest.raises(CommandError) as exc:
            cfg.set("foo", "baz")

    assert "File locked after retries" in str(exc.value)
    assert not tmp_file.exists()


def test_delete_generic_exception_unlinks_tmp(tmp_path: Path) -> None:
    """Test that delete cleans up the temp file after a generic exception."""
    cfg = _cfg(tmp_path)

    def fail_replace(self: Path, target: Path) -> None:
        if self.suffix == ".tmp" and target == cfg._path:
            raise ValueError("boom")
        real_replace(self, target)

    real_replace = Path.replace
    with patch(  # noqa: SIM117
        "pathlib.Path.replace", autospec=True, side_effect=fail_replace
    ):
        with pytest.raises(CommandError) as exc:
            cfg.delete("foo")

    assert "Failed to persist config after deleting" in str(exc.value)
    assert cfg._path is not None
    assert not (cfg._path.with_suffix(".tmp")).exists()


def test_delete_retry_exhaustion_unlinks_tmp(tmp_path: Path) -> None:
    """Test that delete cleans up the temp file after exhausting lock retries."""
    cfg = _cfg(tmp_path)

    with (
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("time.sleep", return_value=None),
    ):
        assert cfg._path is not None
        tmp_file = cfg._path.with_suffix(".tmp")
        tmp_file.touch()

        with pytest.raises(CommandError) as exc:
            cfg.delete("foo")

    assert "File locked after retries" in str(exc.value)
    assert not tmp_file.exists()


def test_set_many_exhausts_retries_and_unlinks_tmp_v2(cfg: Config) -> None:
    """Test set_many cleans up the temp file after exhausting lock retries (v2)."""
    with (
        patch("builtins.open") as mock_open,
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("os.fsync"),
        patch("time.sleep"),
        patch.object(Path, "replace"),
        patch.object(Path, "exists", return_value=True),
        patch.object(Path, "unlink") as mock_unlink,
    ):
        mock_file = MagicMock()
        mock_file.fileno.return_value = 42
        mock_file.__enter__.return_value = mock_file
        mock_open.return_value = mock_file

        with pytest.raises(CommandError, match="File locked after retries"):
            cfg.set_many({"a": "b"})

        mock_unlink.assert_called_once()


def test_set_exhausts_retries_and_unlinks_tmp_v2(cfg: Config) -> None:
    """Test set cleans up the temp file after exhausting lock retries (v2)."""
    with (
        patch("builtins.open") as mock_open,
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("os.fsync"),
        patch("time.sleep"),
        patch.object(Path, "replace"),
        patch.object(Path, "exists", return_value=True),
        patch.object(Path, "unlink") as mock_unlink,
    ):
        mock_file = MagicMock()
        mock_file.fileno.return_value = 42
        mock_file.__enter__.return_value = mock_file
        mock_open.return_value = mock_file

        with pytest.raises(CommandError, match="File locked after retries"):
            cfg.set("a", "b")

        mock_unlink.assert_called_once()


def test_delete_generic_replace_error_cleans_tmp_v2(cfg: Config) -> None:
    """Test that delete cleans up the temp file after a generic exception (v2)."""
    cfg._data = {"a": "b"}

    with (
        patch("builtins.open") as mock_open,
        patch("fcntl.flock"),
        patch("os.fsync"),
        patch.object(Path, "replace", side_effect=Exception("boom")),
        patch.object(Path, "exists", return_value=True),
        patch.object(Path, "unlink") as mock_unlink,
    ):
        mock_file = MagicMock()
        mock_file.fileno.return_value = 42
        mock_file.__enter__.return_value = mock_file
        mock_open.return_value = mock_file

        with pytest.raises(CommandError, match="boom"):
            cfg.delete("a")

        mock_unlink.assert_called_once()


def test_delete_exhausts_retries_and_unlinks_tmp_v2(cfg: Config) -> None:
    """Test that delete cleans up the temp file after exhausting lock retries (v2)."""
    cfg._data = {"a": "b"}

    with (
        patch("builtins.open") as mock_open,
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("os.fsync"),
        patch("time.sleep"),
        patch.object(Path, "replace"),
        patch.object(Path, "exists", return_value=True),
        patch.object(Path, "unlink") as mock_unlink,
    ):
        mock_file = MagicMock()
        mock_file.fileno.return_value = 42
        mock_file.__enter__.return_value = mock_file
        mock_open.return_value = mock_file

        with pytest.raises(CommandError, match="File locked after retries"):
            cfg.delete("a")

        mock_unlink.assert_called_once()


def test_set_many_cleans_up_after_retry_exhaustion_v2(tmp_path: Path) -> None:
    """Test the cleanup branch in set_many after retry exhaustion (v2)."""
    config = Config(MagicMock())
    config._path = tmp_path / "test.env"
    tmp_file = config._path.with_suffix(".tmp")
    tmp_file.touch()

    with (  # noqa: SIM117
        patch("builtins.open"),
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("time.sleep"),
    ):
        with pytest.raises(CommandError, match="File locked after retries"):
            config.set_many({"key": "value"})

    assert not tmp_file.exists()


def test_set_cleans_up_after_retry_exhaustion_v2(tmp_path: Path) -> None:
    """Test the cleanup branch in set after retry exhaustion (v2)."""
    config = Config(MagicMock())
    config._path = tmp_path / "test.env"
    tmp_file = config._path.with_suffix(".tmp")
    tmp_file.touch()

    with (  # noqa: SIM117
        patch("builtins.open"),
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("time.sleep"),
    ):
        with pytest.raises(CommandError, match="File locked after retries"):
            config.set("key", "value")

    assert not tmp_file.exists()


def test_delete_cleans_up_after_generic_exception_v2(tmp_path: Path) -> None:
    """Test the cleanup branch in delete after a generic exception (v2)."""
    config = Config(MagicMock())
    config._path = tmp_path / "test.env"
    config._data = {"key": "value"}
    tmp_file = config._path.with_suffix(".tmp")
    tmp_file.touch()

    with (  # noqa: SIM117
        patch("builtins.open"),
        patch("pathlib.Path.replace", side_effect=ValueError("generic error")),
    ):
        with pytest.raises(CommandError, match="Failed to persist config"):
            config.delete("key")

    assert not tmp_file.exists()


def test_delete_cleans_up_after_retry_exhaustion_v2(tmp_path: Path) -> None:
    """Test the cleanup branch in delete after retry exhaustion (v2)."""
    config = Config(MagicMock())
    config._path = tmp_path / "test.env"
    config._data = {"key": "value"}
    tmp_file = config._path.with_suffix(".tmp")
    tmp_file.touch()

    with (  # noqa: SIM117
        patch("builtins.open"),
        patch("fcntl.flock", side_effect=BlockingIOError),
        patch("time.sleep"),
    ):
        with pytest.raises(CommandError, match="File locked after retries"):
            config.delete("key")

    assert not tmp_file.exists()
