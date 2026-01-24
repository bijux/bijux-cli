# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the config command."""

from __future__ import annotations

import builtins
from collections.abc import Callable
import fcntl
from io import StringIO
from pathlib import Path
import sys
from typing import Any, cast
from unittest.mock import MagicMock, patch

import click
import pytest
import typer
from typer import Context

from bijux_cli.cli.commands.config import import_config
from bijux_cli.cli.commands.config.clear import clear_config
from bijux_cli.cli.commands.config.export import export_config
from bijux_cli.cli.commands.config.get import get_config
from bijux_cli.cli.commands.config.list_cmd import list_config
from bijux_cli.cli.commands.config.load import load_config
from bijux_cli.cli.commands.config.reload import reload_config
from bijux_cli.cli.commands.config.service import config
from bijux_cli.cli.commands.config.set import set_config
from bijux_cli.cli.commands.config.unset import unset_config
from bijux_cli.cli.commands.payloads import (
    ConfigClearPayload,
    ConfigDumpPayload,
    ConfigExportPayload,
    ConfigGetPayload,
    ConfigListPayload,
    ConfigLoadPayload,
    ConfigReloadPayload,
    ConfigSetPayload,
    ConfigUnsetPayload,
)
from bijux_cli.core.enums import ColorMode, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.errors import ConfigError
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError
from bijux_cli.core.precedence import ExecutionPolicy
from bijux_cli.core.runtime import execute_exit_intent


def _raise_exit_intent(*_args: Any, **_kwargs: Any) -> None:
    """Raise an ExitIntentError to stop command execution in tests."""
    intent = ExitIntent(
        code=ExitCode.ERROR,
        stream="stderr",
        payload=None,
        fmt=OutputFormat.JSON,
        pretty=False,
        show_traceback=False,
    )
    raise ExitIntentError(intent)


@pytest.fixture
def mock_flags() -> ExecutionPolicy:
    """Provide a mock execution policy."""
    return ExecutionPolicy(
        output_format=OutputFormat.JSON,
        color=ColorMode.AUTO,
        quiet=False,
        log_level=LogLevel.INFO,
        pretty=True,
        include_runtime=False,
    )


@pytest.fixture
def mock_config_svc() -> MagicMock:
    """Provide a mock of the configuration service."""
    mock = MagicMock()
    return mock


def test_config_callback_no_subcommand(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the main config command callback when no subcommand is invoked."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.service.DIContainer.current"
        ) as mock_current,
        patch("bijux_cli.cli.commands.config.service.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        mock_config_svc.all.return_value = {"key1": "value1"}
        config(ctx)
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigDumpPayload)
        assert payload.entries == {"key1": "value1"}
        assert builder(True).python is not None


def test_clear_config_success(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the successful clearing of the configuration."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.clear.DIContainer.current"
        ) as mock_current,
        patch("bijux_cli.cli.commands.config.clear.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        clear_config(ctx)
        mock_config_svc.clear.assert_called_once()
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigClearPayload)
        assert payload.status == "cleared"
        assert builder(True).python is not None


def test_clear_config_fail(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the failure path when clearing the configuration."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.clear.DIContainer.current"
        ) as mock_current,
        patch(
            "bijux_cli.cli.commands.config.clear.emit_error_with_policy"
        ) as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.clear.side_effect = Exception("error")
        with pytest.raises(ExitIntentError) as exc:
            clear_config(ctx)
        with pytest.raises(typer.Exit):
            execute_exit_intent(exc.value.intent)
        mock_emit.assert_called()


def test_export_config_stdout(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test exporting the configuration to stdout."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.export.DIContainer.current"
        ) as mock_current,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        export_config(ctx, "-", "json")
        mock_config_svc.export.assert_called_with("-", "json")


def test_export_config_file(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test exporting the configuration to a file."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.export.DIContainer.current"
        ) as mock_current,
        patch("bijux_cli.cli.commands.config.export.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        export_config(ctx, "file", "json")
        mock_config_svc.export.assert_called_with("file", "json")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigExportPayload)
        assert payload.status == "exported"
        assert builder(True).python is not None


def test_get_config_success(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test successfully getting a configuration value."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.get.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.get.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.get.return_value = "value"
        get_config(ctx, "key")
        mock_config_svc.get.assert_called_with("key")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigGetPayload)
        assert payload.value == "value"
        assert builder(True).python is not None


def test_list_config_success(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test successfully listing all configuration keys."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.list_cmd.DIContainer.current"
        ) as mock_current,
        patch("bijux_cli.cli.commands.config.list_cmd.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.list_keys.return_value = ["key1", "key2"]
        list_config(ctx)
        mock_config_svc.list_keys.assert_called_once()
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigListPayload)
        assert payload.items == [{"key": "key1"}, {"key": "key2"}]
        assert builder(True).python is not None


def test_load_config_success(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test successfully loading configuration from a file."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.load.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.load.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        load_config(ctx, "path")
        mock_config_svc.load.assert_called_with("path")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigLoadPayload)
        assert payload.status == "loaded"
        assert payload.file == "path"
        assert builder(True).python is not None


def test_load_config_exception(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the failure path when loading configuration from a file."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.load.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.load.emit_error_with_policy") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.load.side_effect = Exception("error")
        with pytest.raises(ExitIntentError):
            load_config(ctx, "path")
        mock_emit.assert_called()


def test_reload_config_success(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the successful reloading of the configuration."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.reload.DIContainer.current"
        ) as mock_current,
        patch("bijux_cli.cli.commands.config.reload.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        reload_config(ctx)
        mock_config_svc.reload.assert_called_once()
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigReloadPayload)
        assert payload.status == "reloaded"
        assert builder(True).python is not None


def test_reload_config_exception(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the failure path when reloading the configuration."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.reload.DIContainer.current"
        ) as mock_current,
        patch(
            "bijux_cli.cli.commands.config.reload.emit_error_with_policy"
        ) as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.reload.side_effect = Exception("error")
        with pytest.raises(ExitIntentError):
            reload_config(ctx)
        mock_emit.assert_called()


def test_set_config_arg(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test setting a configuration value from a command-line argument."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        set_config(ctx, "key=value")
        mock_config_svc.set.assert_called_with("key", "value")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigSetPayload)
        assert payload.status == "updated"
        assert payload.key == "key"
        assert payload.value == "value"
        assert builder(True).python is not None


def test_set_config_stdin(
    mock_flags: ExecutionPolicy,
    mock_config_svc: MagicMock,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test setting a configuration value from stdin."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.new_run_command"),
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        monkeypatch.setattr(sys, "stdin", StringIO("key=value\n"))
        monkeypatch.setattr(sys.stdin, "isatty", lambda: False)
        set_config(ctx, None)
        mock_config_svc.set.assert_called_with("key", "value")


def test_set_config_empty_key(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with an empty key fails."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.emit_error_with_policy") as mock_emit,
    ):
        mock_emit.side_effect = _raise_exit_intent
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(ExitIntentError):
            set_config(ctx, "=value")
        mock_emit.assert_called()


def test_set_config_non_ascii(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with non-ASCII characters fails."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.emit_error_with_policy") as mock_emit,
    ):
        mock_emit.side_effect = _raise_exit_intent
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(ExitIntentError):
            set_config(ctx, "key=value©")
        mock_emit.assert_called()


def test_set_config_control_char(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with a control character fails."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.emit_error_with_policy") as mock_emit,
    ):
        mock_emit.side_effect = _raise_exit_intent
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(ExitIntentError):
            set_config(ctx, "key=value\x07")
        mock_emit.assert_called()


def test_set_config_invalid_key(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with an invalid key format fails."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.emit_error_with_policy") as mock_emit,
    ):
        mock_emit.side_effect = _raise_exit_intent
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(ExitIntentError):
            set_config(ctx, "invalid-key=value")
        mock_emit.assert_called()


def test_set_config_exception(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the failure path when the config service 'set' method raises an exception."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.emit_error_with_policy") as mock_emit,
    ):
        mock_emit.side_effect = _raise_exit_intent
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.set.side_effect = Exception("error")
        with pytest.raises(ExitIntentError):
            set_config(ctx, "key=value")
        mock_emit.assert_called()


def test_unset_config_success(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the successful unsetting of a configuration key."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.unset.DIContainer.current"
        ) as mock_current,
        patch("bijux_cli.cli.commands.config.unset.new_run_command") as mock_new_run,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        unset_config(ctx, "key")
        mock_config_svc.unset.assert_called_with("key")
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)
        assert isinstance(payload, ConfigUnsetPayload)
        assert payload.status == "deleted"
        assert payload.key == "key"
        assert builder(True).python is not None


def test_unset_config_key_error(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that unsetting a non-existent key is handled correctly."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.unset.DIContainer.current"
        ) as mock_current,
        patch(
            "bijux_cli.cli.commands.config.unset.emit_error_with_policy"
        ) as mock_emit,
    ):
        mock_emit.side_effect = _raise_exit_intent
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.unset.side_effect = KeyError("key")
        with pytest.raises(ExitIntentError):
            unset_config(ctx, "key")
        mock_emit.assert_called()


def test_unset_config_exception(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the failure path when the config service 'unset' raises an exception."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.unset.DIContainer.current"
        ) as mock_current,
        patch(
            "bijux_cli.cli.commands.config.unset.emit_error_with_policy"
        ) as mock_emit,
    ):
        mock_emit.side_effect = _raise_exit_intent
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.unset.side_effect = Exception("error")
        with pytest.raises(ExitIntentError):
            unset_config(ctx, "key")
        mock_emit.assert_called()


def test_import_config(mock_flags: ExecutionPolicy) -> None:
    """Test that the 'import' command correctly calls the 'load_config' function."""
    with patch("bijux_cli.cli.commands.config.load_config") as mock_load:
        ctx = Context(MagicMock())
        import_config(ctx, "path")
        mock_load.assert_called_with(ctx, "path")


def test_export_config_command_error(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that a ConfigError during export is handled correctly."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.export.DIContainer.current"
        ) as mock_current,
        patch(
            "bijux_cli.cli.commands.config.export.emit_error_with_policy"
        ) as mock_emit,
    ):
        mock_emit.side_effect = typer.Exit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.export.side_effect = ConfigError("error")
        with pytest.raises(typer.Exit):
            export_config(ctx, "file", "json")
        mock_emit.assert_called()


def test_export_config_exception(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that a generic Exception during export is propagated."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.export.DIContainer.current"
        ) as mock_current,
        patch("bijux_cli.cli.commands.config.export.emit_error_with_policy"),
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.export.side_effect = Exception("error")
        with pytest.raises(Exception, match="error"):
            export_config(ctx, "file", "json")


def test_get_config_not_found(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that a ConfigError when getting a non-existent key is handled."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.get.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.get.emit_error_with_policy") as mock_emit,
    ):
        mock_emit.side_effect = typer.Exit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.get.side_effect = ConfigError("Config key not found: key")
        with pytest.raises(typer.Exit):
            get_config(ctx, "key")
        mock_emit.assert_called()


def test_get_config_exception(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that a generic exception when getting a config value is propagated."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.get.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.get.emit_error_with_policy"),
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.get.side_effect = Exception("error")
        with pytest.raises(Exception, match="error"):
            get_config(ctx, "key")


def test_list_config_exception(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test the failure path when listing configuration keys."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch(
            "bijux_cli.cli.commands.config.list_cmd.DIContainer.current"
        ) as mock_current,
        patch(
            "bijux_cli.cli.commands.config.list_cmd.emit_error_with_policy"
        ) as mock_emit,
    ):
        mock_emit.side_effect = typer.Exit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        mock_config_svc.list_keys.side_effect = Exception("error")
        with pytest.raises(typer.Exit):
            list_config(ctx)
        mock_emit.assert_called()


def test_set_config_no_arg_tty(
    mock_flags: ExecutionPolicy,
    mock_config_svc: MagicMock,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that setting a value with no argument on a TTY fails."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.emit_error_with_policy") as mock_emit,
    ):
        mock_emit.side_effect = typer.Exit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        monkeypatch.setattr(sys.stdin, "isatty", lambda: True)
        with pytest.raises(typer.Exit):
            set_config(ctx, None)
        mock_emit.assert_called()


def test_set_config_invalid_pair(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that setting a value with an invalid pair format fails."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.emit_error_with_policy") as mock_emit,
    ):
        mock_emit.side_effect = typer.Exit
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        with pytest.raises(typer.Exit):
            set_config(ctx, "key")
        mock_emit.assert_called()


def test_set_config_stdin_escaped(
    mock_flags: ExecutionPolicy,
    mock_config_svc: MagicMock,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that escaped characters from stdin are correctly handled."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.set.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.set.new_run_command"),
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        ctx = Context(MagicMock())
        monkeypatch.setattr(sys, "stdin", StringIO('key="a value with a \\" quote"\n'))
        monkeypatch.setattr(sys.stdin, "isatty", lambda: False)
        set_config(ctx, None)
        mock_config_svc.set.assert_called_with("key", 'a value with a " quote')


def test_get_config_other_command_error(
    mock_flags: ExecutionPolicy, mock_config_svc: MagicMock
) -> None:
    """Test that a generic ConfigError during get is handled correctly."""
    from bijux_cli.cli.commands.config.get import get_config

    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=mock_flags,
        ),
        patch("bijux_cli.cli.commands.config.get.DIContainer.current") as mock_current,
        patch("bijux_cli.cli.commands.config.get.emit_error_with_policy") as mock_emit,
    ):
        mock_current.return_value.resolve.return_value = mock_config_svc
        mock_config_svc.get.side_effect = ConfigError("boom!")
        mock_emit.side_effect = typer.Exit
        ctx = Context(MagicMock())
        with pytest.raises(typer.Exit):
            get_config(ctx, "anykey")
        mock_emit.assert_called_once()
        name, kwargs = mock_emit.call_args
        assert kwargs.get("failure") == "get_failed"
        assert (
            "Failed to get config: boom!" in kwargs.get("msg", "")
            or "Failed to get config: boom!" in name[0]
        )


class DummyCmd(click.Command):
    """Minimal Click/Typer command for Context construction in tests."""

    def __init__(self) -> None:
        super().__init__(name="dummy")
        self.allow_extra_args = True
        self.allow_interspersed_args = True
        self.ignore_unknown_options = True

    def invoke(self, ctx: click.Context) -> Any:
        return None


def test_config_root_with_subcommand_skips_execution(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that the main config callback returns early if a subcommand is invoked."""
    fake_ctx = Context(
        command=DummyCmd(),
        allow_extra_args=True,
        ignore_unknown_options=True,
    )
    fake_ctx.invoked_subcommand = "something"

    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: (_ for _ in ()).throw(
            AssertionError("current_execution_policy should not run")
        ),
    )
    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.service.DIContainer.current",
        lambda: (_ for _ in ()).throw(
            AssertionError("DIContainer.current should not run")
        ),
    )
    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.service.new_run_command",
        lambda *args, **kwargs: (_ for _ in ()).throw(
            AssertionError("new_run_command should not run")
        ),
    )

    result = cast(Callable[..., Any], config)(fake_ctx)
    assert result is None


class DummySvc:
    """A mock configuration service for testing."""

    def set(self, key: str, val: str) -> None:
        """Mock the set method."""


def make_ctx() -> Context:
    """Build a Typer Context with a dummy command allowing extra arguments."""
    return Context(
        command=DummyCmd(),
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
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=False,
        ),
    )

    with pytest.raises(ExitIntentError) as exc:
        set_config(make_ctx(), "key=value")
    payload = cast(dict[str, Any], exc.value.intent.payload)
    assert "Non-ASCII" in payload["error"]


def test_posix_lock_failure(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    """Test that a failure to acquire a POSIX file lock is handled correctly."""
    cfg = tmp_path / "cfg"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=False,
        ),
    )

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.set.DIContainer.current", FakeContainer
    )

    monkeypatch.setattr(
        fcntl, "flock", lambda fh, flags: (_ for _ in ()).throw(OSError("locked"))
    )

    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.set.emit_error_with_policy",
        lambda msg, **kwargs: (_ for _ in ()).throw(typer.Exit(1)),
    )

    with pytest.raises(typer.Exit) as exc:
        set_config(make_ctx(), "key=value")
    assert exc.value.exit_code == 1


def test_posix_lock_success_and_run(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test the successful acquisition and release of a POSIX file lock."""
    cfg = tmp_path / "cfg2"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=False,
            include_runtime=True,
        ),
    )

    class DummySvc:
        last: tuple[str, str] | None = None

        def set(self, key: str, val: str) -> None:
            self.last = (key, val)

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.set.DIContainer.current", FakeContainer
    )

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.set.new_run_command",
        lambda **kw: captured.update(kw),
    )

    set_config(make_ctx(), "foo=bar")

    assert "payload_builder" in captured
    builder = captured["payload_builder"]
    no_rt = builder(False)
    assert isinstance(no_rt, ConfigSetPayload)
    assert no_rt.status == "updated"
    assert no_rt.key == "foo"
    assert no_rt.value == "bar"
    with_rt = builder(True)
    assert with_rt.python is not None
    assert with_rt.platform is not None


def test_posix_lock_import_failure_skips_lock(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that a failure to import 'fcntl' skips the locking mechanism."""
    cfg = tmp_path / "cfg"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=False,
        ),
    )

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.set.DIContainer.current",
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
        "bijux_cli.cli.commands.config.set.new_run_command",
        lambda **kwargs: captured.update(kwargs),
    )

    ctx = Context(
        DummyCmd(),
        allow_extra_args=True,
        allow_interspersed_args=True,
        ignore_unknown_options=True,
    )

    set_config(ctx, "abc=123")

    assert "payload_builder" in captured
    payload = captured["payload_builder"](False)
    assert isinstance(payload, ConfigSetPayload)
    assert payload.status == "updated"
    assert payload.key == "abc"
    assert payload.value == "123"


def test_posix_unlock_failure_is_ignored(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that an error during file unlock is ignored and does not crash."""
    cfg = tmp_path / "cfg_unlock"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=False,
            include_runtime=True,
        ),
    )

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.set.DIContainer.current",
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
        "bijux_cli.cli.commands.config.set.new_run_command",
        lambda **kw: captured.update(kw),
    )

    ctx = Context(
        DummyCmd(),
        allow_extra_args=True,
        allow_interspersed_args=True,
        ignore_unknown_options=True,
    )

    set_config(ctx, "foo=bar")

    assert "payload_builder" in captured
    payload = captured["payload_builder"](False)
    assert isinstance(payload, ConfigSetPayload)
    assert payload.status == "updated"
    assert payload.key == "foo"
    assert payload.value == "bar"


def test_non_posix_skips_file_lock_block(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that the POSIX file lock block is skipped on non-POSIX systems."""
    cfg = tmp_path / "cfg_win"
    cfg.write_text("")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=False,
            include_runtime=True,
        ),
    )

    class FakeContainer:
        def resolve(self, _: Any) -> DummySvc:
            return DummySvc()

    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.set.DIContainer.current",
        staticmethod(lambda: FakeContainer()),
    )

    import os

    monkeypatch.setattr(os, "name", "nt")

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.cli.commands.config.set.new_run_command",
        lambda **kw: captured.update(kw),
    )

    ctx = Context(
        DummyCmd(),
        allow_extra_args=True,
        allow_interspersed_args=True,
        ignore_unknown_options=True,
    )

    set_config(ctx, "winkey=winval")

    assert "payload_builder" in captured
    out = captured["payload_builder"](False)
    assert isinstance(out, ConfigSetPayload)
    assert out.status == "updated"
    assert out.key == "winkey"
    assert out.value == "winval"
