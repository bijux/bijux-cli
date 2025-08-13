# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the config command."""

from __future__ import annotations

import builtins
import fcntl
from io import StringIO
from pathlib import Path
import sys
from typing import Any
from unittest.mock import MagicMock, patch

import pytest
from typer import Context

from bijux_cli.commands.config import import_config
from bijux_cli.commands.config.clear import clear_config
from bijux_cli.commands.config.export import export_config
from bijux_cli.commands.config.get import get_config
from bijux_cli.commands.config.list_cmd import list_config
from bijux_cli.commands.config.load import load_config
from bijux_cli.commands.config.reload import reload_config
from bijux_cli.commands.config.service import config
from bijux_cli.commands.config.set import set_config
from bijux_cli.commands.config.unset import unset_config
from bijux_cli.core.exceptions import CommandError


@pytest.fixture
def mock_flags() -> dict[str, Any]:
    """Provide a dictionary of mock global flags."""
    return {
        "quiet": False,
        "verbose": False,
        "format": "json",
        "pretty": True,
        "debug": False,
    }


@pytest.fixture
def mock_config_svc() -> MagicMock:
    """Provide a mock of the configuration service."""
    mock = MagicMock()
    return mock


def test_config_callback_no_subcommand(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the main config command callback when no subcommand is invoked."""
    with (
        patch(
            "bijux_cli.commands.config.service.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.service.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.service.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        mock_config_svc.all.return_value = {"key1": "value1"}
        config(ctx)
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"key1": "value1"}
        assert "python" in builder(True)


def test_clear_config_success(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the successful clearing of the configuration."""
    with (
        patch(
            "bijux_cli.commands.config.clear.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.clear.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.clear.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        clear_config(ctx)
        mock_config_svc.clear.assert_called_once()
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"status": "cleared"}
        assert "python" in builder(True)


def test_clear_config_fail(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the failure path when clearing the configuration."""
    with (
        patch(
            "bijux_cli.commands.config.clear.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.clear.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.clear.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.clear.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            clear_config(ctx)
        mock_emit.assert_called()


def test_export_config_stdout(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test exporting the configuration to stdout."""
    with (
        patch(
            "bijux_cli.commands.config.export.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.export.DIContainer.current") as mock_current,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        export_config(ctx, "-", None)  # type: ignore[arg-type]
        mock_config_svc.export.assert_called_with("-", None)


def test_export_config_file(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test exporting the configuration to a file."""
    with (
        patch(
            "bijux_cli.commands.config.export.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.export.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.export.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        export_config(ctx, "file", "json")
        mock_config_svc.export.assert_called_with("file", "json")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False)["status"] == "exported"
        assert "python" in builder(True)


def test_get_config_success(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test successfully getting a configuration value."""
    with (
        patch(
            "bijux_cli.commands.config.get.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.get.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.get.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.get.return_value = "value"
        get_config(ctx, "key")
        mock_config_svc.get.assert_called_with("key")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"value": "value"}
        assert "python" in builder(True)


def test_list_config_success(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test successfully listing all configuration keys."""
    with (
        patch(
            "bijux_cli.commands.config.list_cmd.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.list_cmd.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.list_cmd.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.list_keys.return_value = ["key1", "key2"]
        list_config(ctx)
        mock_config_svc.list_keys.assert_called_once()
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"items": [{"key": "key1"}, {"key": "key2"}]}
        assert "python" in builder(True)


def test_load_config_success(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test successfully loading configuration from a file."""
    with (
        patch(
            "bijux_cli.commands.config.load.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.load.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.load.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        load_config(ctx, "path")
        mock_config_svc.load.assert_called_with("path")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"status": "loaded", "file": "path"}
        assert "python" in builder(True)


def test_load_config_exception(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the failure path when loading configuration from a file."""
    with (
        patch(
            "bijux_cli.commands.config.load.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.load.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.load.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.load.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            load_config(ctx, "path")
        mock_emit.assert_called()


def test_reload_config_success(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the successful reloading of the configuration."""
    with (
        patch(
            "bijux_cli.commands.config.reload.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.reload.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.reload.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        reload_config(ctx)
        mock_config_svc.reload.assert_called_once()
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"status": "reloaded"}
        assert "python" in builder(True)


def test_reload_config_exception(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the failure path when reloading the configuration."""
    with (
        patch(
            "bijux_cli.commands.config.reload.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.reload.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.reload.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.reload.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            reload_config(ctx)
        mock_emit.assert_called()


def test_set_config_arg(mock_flags: dict[str, Any], mock_config_svc: MagicMock) -> None:
    """Test setting a configuration value from a command-line argument."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        set_config(ctx, "key=value")
        mock_config_svc.set.assert_called_with("key", "value")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"status": "updated", "key": "key", "value": "value"}
        assert "python" in builder(True)


def test_set_config_stdin(
    mock_flags: dict[str, Any],
    mock_config_svc: MagicMock,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test setting a configuration value from stdin."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.new_run_command"),
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        monkeypatch.setattr(sys, "stdin", StringIO("key=value\n"))
        monkeypatch.setattr(sys.stdin, "isatty", lambda: False)
        set_config(ctx, None)
        mock_config_svc.set.assert_called_with("key", "value")


def test_set_config_empty_key(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with an empty key fails."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(SystemExit):
            set_config(ctx, "=value")
        mock_emit.assert_called()


def test_set_config_non_ascii(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with non-ASCII characters fails."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(SystemExit):
            set_config(ctx, "key=value©")
        mock_emit.assert_called()


def test_set_config_control_char(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with a control character fails."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(SystemExit):
            set_config(ctx, "key=value\x07")
        mock_emit.assert_called()


def test_set_config_invalid_key(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with an invalid key format fails."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(SystemExit):
            set_config(ctx, "invalid-key=value")
        mock_emit.assert_called()


def test_set_config_exception(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the failure path when the config service 'set' method raises an exception."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.set.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            set_config(ctx, "key=value")
        mock_emit.assert_called()


def test_unset_config_success(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the successful unsetting of a configuration key."""
    with (
        patch(
            "bijux_cli.commands.config.unset.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.unset.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.unset.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        unset_config(ctx, "key")
        mock_config_svc.unset.assert_called_with("key")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"status": "deleted", "key": "key"}
        assert "python" in builder(True)


def test_unset_config_key_error(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that unsetting a non-existent key is handled correctly."""
    with (
        patch(
            "bijux_cli.commands.config.unset.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.unset.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.unset.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.unset.side_effect = KeyError("key")
        with pytest.raises(SystemExit):
            unset_config(ctx, "key")
        mock_emit.assert_called()


def test_unset_config_exception(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the failure path when the config service 'unset' raises an exception."""
    with (
        patch(
            "bijux_cli.commands.config.unset.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.unset.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.unset.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.unset.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            unset_config(ctx, "key")
        mock_emit.assert_called()


def test_import_config(mock_flags: dict[str, Any]) -> None:
    """Test that the 'import' command correctly calls the 'load_config' function."""
    with patch("bijux_cli.commands.config.load_config") as mock_load:
        ctx = Context(MagicMock())
        import_config(ctx, "path")
        mock_load.assert_called_with(ctx, "path")


def test_export_config_command_error(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that a CommandError during export is handled correctly."""
    with (
        patch(
            "bijux_cli.commands.config.export.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.export.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.export.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.export.side_effect = CommandError("error")
        with pytest.raises(SystemExit):
            export_config(ctx, "file", None)  # type: ignore[arg-type]
        mock_emit.assert_called()


def test_export_config_exception(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that a generic Exception during export is propagated."""
    with (
        patch(
            "bijux_cli.commands.config.export.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.export.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.export.emit_error_and_exit"),
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.export.side_effect = Exception("error")
        with pytest.raises(Exception, match="error"):
            export_config(ctx, "file", None)  # type: ignore[arg-type]


def test_get_config_not_found(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that a CommandError when getting a non-existent key is handled."""
    with (
        patch(
            "bijux_cli.commands.config.get.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.get.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.get.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.get.side_effect = CommandError("Config key not found: key")
        with pytest.raises(SystemExit):
            get_config(ctx, "key")
        mock_emit.assert_called()


def test_get_config_exception(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that a generic exception when getting a config value is propagated."""
    with (
        patch(
            "bijux_cli.commands.config.get.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.get.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.get.emit_error_and_exit"),
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.get.side_effect = Exception("error")
        with pytest.raises(Exception, match="error"):
            get_config(ctx, "key")


def test_list_config_exception(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test the failure path when listing configuration keys."""
    with (
        patch(
            "bijux_cli.commands.config.list_cmd.parse_global_flags",
            return_value=mock_flags,
        ),
        patch("bijux_cli.commands.config.list_cmd.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.list_cmd.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.list_keys.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            list_config(ctx)
        mock_emit.assert_called()


def test_set_config_no_arg_tty(
    mock_flags: dict[str, Any],
    mock_config_svc: MagicMock,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that setting a value with no argument on a TTY fails."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        monkeypatch.setattr(sys.stdin, "isatty", lambda: True)
        with pytest.raises(SystemExit):
            set_config(ctx, None)
        mock_emit.assert_called()


def test_set_config_invalid_pair(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with an invalid pair format fails."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(SystemExit):
            set_config(ctx, "key")
        mock_emit.assert_called()


def test_set_config_stdin_escaped(
    mock_flags: dict[str, Any],
    mock_config_svc: MagicMock,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that escaped characters from stdin are correctly handled."""
    with (
        patch(
            "bijux_cli.commands.config.set.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.set.new_run_command"),
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        monkeypatch.setattr(sys, "stdin", StringIO('key="a value with a \\" quote"\n'))
        monkeypatch.setattr(sys.stdin, "isatty", lambda: False)
        set_config(ctx, None)
        mock_config_svc.set.assert_called_with("key", 'a value with a " quote')


def test_get_config_other_command_error(
    mock_flags: dict[str, Any], mock_config_svc: MagicMock
) -> None:
    """Test that a generic CommandError during get is handled correctly."""
    from bijux_cli.commands.config.get import get_config

    with (
        patch(
            "bijux_cli.commands.config.get.parse_global_flags", return_value=mock_flags
        ),
        patch("bijux_cli.commands.config.get.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.config.get.emit_error_and_exit") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        mock_config_svc.get.side_effect = CommandError("boom!")
        mock_emit.side_effect = SystemExit
        ctx = Context(MagicMock())
        with pytest.raises(SystemExit):
            get_config(ctx, "anykey")
        mock_emit.assert_called_once()
        name, kwargs = mock_emit.call_args
        assert kwargs.get("failure") == "get_failed"
        assert (
            "Failed to get config: boom!" in kwargs.get("msg", "")
            or "Failed to get config: boom!" in name[0]
        )


class DummyCmd:
    """A mock Click/Typer Command with necessary flags for tests."""

    allow_extra_args = True
    allow_interspersed_args = True


def test_config_root_with_subcommand_skips_execution(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that the main config callback returns early if a subcommand is invoked."""
    fake_ctx = Context(
        command=DummyCmd(),  # type: ignore[arg-type]
        allow_extra_args=True,
        ignore_unknown_options=True,
    )
    fake_ctx.invoked_subcommand = "something"

    monkeypatch.setattr(
        "bijux_cli.commands.config.service.parse_global_flags",
        lambda: (_ for _ in ()).throw(
            AssertionError("parse_global_flags should not run")
        ),
    )
    monkeypatch.setattr(
        "bijux_cli.commands.config.service.DIContainer.current",
        lambda: (_ for _ in ()).throw(
            AssertionError("DIContainer.current should not run")
        ),
    )
    monkeypatch.setattr(
        "bijux_cli.commands.config.service.new_run_command",
        lambda *args, **kwargs: (_ for _ in ()).throw(
            AssertionError("new_run_command should not run")
        ),
    )

    result = config(fake_ctx)  # type: ignore[func-returns-value]
    assert result is None


class DummySvc:
    """A mock configuration service for testing."""

    def set(self, key: str, val: str) -> None:
        """Mock the set method."""


def make_ctx() -> Context:
    """Build a Typer Context with a dummy command allowing extra arguments."""
    return Context(
        command=DummyCmd(),  # type: ignore[arg-type]
        allow_extra_args=True,
        ignore_unknown_options=True,
    )


def test_non_ascii_config_path_triggers_error(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that a non-ASCII config path from the environment results in an error."""
    bad_path = tmp_path / "päth"
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(bad_path))

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.parse_global_flags",
        lambda: {
            "quiet": False,
            "verbose": False,
            "format": "json",
            "pretty": True,
            "debug": False,
        },
    )

    called: dict[str, Any] = {}

    def fake_emit(msg: str, **kwargs: Any) -> None:
        called["msg"] = msg
        raise SystemExit(3)

    monkeypatch.setattr("bijux_cli.commands.config.set.emit_error_and_exit", fake_emit)

    with pytest.raises(SystemExit) as exc:
        set_config(make_ctx(), "key=value")
    assert exc.value.code == 3
    assert "Non-ASCII characters in config path" in called["msg"]


def test_posix_lock_failure(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    """Test that a failure to acquire a POSIX file lock is handled correctly."""
    cfg = tmp_path / "cfg"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.parse_global_flags",
        lambda: {
            "quiet": False,
            "verbose": False,
            "format": "json",
            "pretty": True,
            "debug": False,
        },
    )

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.DIContainer.current", FakeContainer
    )

    monkeypatch.setattr(
        fcntl, "flock", lambda fh, flags: (_ for _ in ()).throw(OSError("locked"))
    )

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.emit_error_and_exit",
        lambda msg, **kwargs: (_ for _ in ()).throw(SystemExit(1)),
    )

    with pytest.raises(SystemExit) as exc:
        set_config(make_ctx(), "key=value")
    assert exc.value.code == 1


def test_posix_lock_success_and_run(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test the successful acquisition and release of a POSIX file lock."""
    cfg = tmp_path / "cfg2"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.parse_global_flags",
        lambda: {
            "quiet": False,
            "verbose": True,
            "format": "json",
            "pretty": False,
            "debug": False,
        },
    )

    class DummySvc:
        last: tuple[str, str] | None = None

        def set(self, key: str, val: str) -> None:
            self.last = (key, val)

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()  # pyright: ignore[reportReturnType]

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.DIContainer.current", FakeContainer
    )

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.commands.config.set.new_run_command",
        lambda **kw: captured.update(kw),
    )

    set_config(make_ctx(), "foo=bar")

    assert "payload_builder" in captured
    builder = captured["payload_builder"]
    no_rt = builder(False)
    assert no_rt == {"status": "updated", "key": "foo", "value": "bar"}
    with_rt = builder(True)
    assert "python" in with_rt
    assert "platform" in with_rt


def test_posix_lock_import_failure_skips_lock(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that a failure to import 'fcntl' skips the locking mechanism."""
    cfg = tmp_path / "cfg"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.parse_global_flags",
        lambda: {
            "quiet": False,
            "verbose": False,
            "format": "json",
            "pretty": True,
            "debug": False,
        },
    )

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.DIContainer.current",
        staticmethod(lambda: FakeContainer()),
    )

    real_import = builtins.__import__

    def fake_import(
        name: str,
        globals_: dict[str, Any] | None = None,
        locals_: dict[str, Any] | None = None,
        fromlist: tuple[str, ...] = (),
        level: int = 0,
    ) -> Any:
        if name == "fcntl":
            raise ImportError("no fcntl")
        return real_import(name, globals_, locals_, fromlist, level)

    monkeypatch.setattr(builtins, "__import__", fake_import)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.commands.config.set.new_run_command",
        lambda **kwargs: captured.update(kwargs),
    )

    ctx = Context(
        DummyCmd(),  # type: ignore[arg-type]
        allow_extra_args=True,
        allow_interspersed_args=True,
        ignore_unknown_options=True,
    )

    set_config(ctx, "abc=123")

    assert "payload_builder" in captured
    payload = captured["payload_builder"](False)
    assert payload == {"status": "updated", "key": "abc", "value": "123"}


def test_posix_unlock_failure_is_ignored(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that an error during file unlock is ignored and does not crash."""
    cfg = tmp_path / "cfg_unlock"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.parse_global_flags",
        lambda: {
            "quiet": False,
            "verbose": True,
            "format": "json",
            "pretty": False,
            "debug": False,
        },
    )

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.DIContainer.current",
        staticmethod(lambda: FakeContainer()),
    )

    calls: dict[str, int] = {"n": 0}

    def fake_flock(fh: Any, flags: int) -> None:
        calls["n"] += 1
        if calls["n"] == 2:
            raise RuntimeError("unlock‐oops")
        return None

    monkeypatch.setattr(fcntl, "flock", fake_flock)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.commands.config.set.new_run_command",
        lambda **kw: captured.update(kw),
    )

    ctx = Context(
        DummyCmd(),  # type: ignore[arg-type]
        allow_extra_args=True,
        allow_interspersed_args=True,
        ignore_unknown_options=True,
    )

    set_config(ctx, "foo=bar")

    assert "payload_builder" in captured
    payload = captured["payload_builder"](False)
    assert payload == {"status": "updated", "key": "foo", "value": "bar"}


def test_non_posix_skips_file_lock_block(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that the POSIX file lock block is skipped on non-POSIX systems."""
    cfg = tmp_path / "cfg_win"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.parse_global_flags",
        lambda: {
            "quiet": False,
            "verbose": True,
            "format": "json",
            "pretty": False,
            "debug": False,
        },
    )

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.commands.config.set.DIContainer.current",
        staticmethod(lambda: FakeContainer()),
    )

    import os

    monkeypatch.setattr(os, "name", "nt")

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.commands.config.set.new_run_command",
        lambda **kw: captured.update(kw),
    )

    ctx = Context(
        DummyCmd(),  # type: ignore[arg-type]
        allow_extra_args=True,
        allow_interspersed_args=True,
        ignore_unknown_options=True,
    )

    set_config(ctx, "winkey=winval")

    assert "payload_builder" in captured
    out = captured["payload_builder"](False)
    assert out == {"status": "updated", "key": "winkey", "value": "winval"}
