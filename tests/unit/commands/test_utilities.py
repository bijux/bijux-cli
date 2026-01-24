# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for CLI emit/validation/output helpers."""

from __future__ import annotations

import os
from pathlib import Path
import sys
import types
from typing import Any, cast
from unittest.mock import MagicMock, patch

import pytest
import typer

from bijux_cli.cli.core.command import new_run_command, raise_exit_intent
from bijux_cli.cli.core.validation import (
    ascii_safe,
    contains_non_ascii_env,
    normalize_format,
    validate_common_flags,
    validate_env_file_if_present,
)
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ColorMode, ErrorType, LogLevel, OutputFormat
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.core.precedence import ExecutionPolicy
from bijux_cli.core.runtime import execute_exit_intent
from bijux_cli.services.history.contracts import HistoryProtocol


@pytest.fixture
def mock_di(monkeypatch: pytest.MonkeyPatch) -> types.SimpleNamespace:
    """Provide a stub DI container that can be easily manipulated in tests."""
    stub = types.SimpleNamespace()

    def _resolve_mock(*args: Any, **kwargs: Any) -> MagicMock:
        return MagicMock()

    stub.resolve = MagicMock(side_effect=_resolve_mock)
    monkeypatch.setattr(DIContainer, "current", staticmethod(lambda: stub))
    return stub


def test_ascii_safe_str_ascii() -> None:
    """Test ascii_safe with a pure ASCII string."""
    assert ascii_safe("abc", "field") == "abc"


def test_ascii_safe_str_non_ascii() -> None:
    """Test ascii_safe with a non-ASCII string, expecting replacement."""
    assert ascii_safe("a\u00a9b", "field") == "a?b"


def test_ascii_safe_str_control_allowed() -> None:
    """Test ascii_safe with allowed control characters."""
    assert ascii_safe("a\nb\r\tc", "field") == "a\nb\r\tc"


def test_ascii_safe_str_control_other() -> None:
    """Test ascii_safe with disallowed control characters, expecting replacement."""
    assert ascii_safe("a\x07b", "field") == "a?b"


def test_ascii_safe_non_str() -> None:
    """Test ascii_safe with a non-string input, expecting string conversion."""
    assert ascii_safe(123, "field") == "123"


def test_ascii_safe_empty() -> None:
    """Test ascii_safe with an empty string."""
    assert ascii_safe("", "field") == ""


def test_normalize_format_none() -> None:
    """Test normalize_format with None input."""
    assert normalize_format(None) is None


def test_normalize_format_empty() -> None:
    """Test normalize_format with an empty string."""
    assert normalize_format("") is None


def test_normalize_format_whitespace() -> None:
    """Test normalize_format with leading/trailing whitespace."""
    assert normalize_format(" json ") == OutputFormat.JSON


def test_normalize_format_upper() -> None:
    """Test normalize_format with an uppercase string."""
    assert normalize_format("YAML") == OutputFormat.YAML


def test_contains_non_ascii_env_no_config(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test non-ASCII check when BIJUXCLI_CONFIG is not set."""
    monkeypatch.delenv("BIJUXCLI_CONFIG", raising=False)
    monkeypatch.setattr(os, "environ", {"BIJUXCLI_OTHER": "ascii"})
    assert not contains_non_ascii_env()


@patch.dict(os.environ, {"BIJUXCLI_CONFIG": "non\u00a9ascii"})
def test_contains_non_ascii_env_config_non_ascii() -> None:
    """Test non-ASCII check when BIJUXCLI_CONFIG itself contains non-ASCII."""
    assert contains_non_ascii_env()


@patch.dict(os.environ, {"BIJUXCLI_CONFIG": "/path/to/config"})
@patch.object(Path, "exists", return_value=True)
@patch.object(
    Path, "read_text", side_effect=UnicodeDecodeError("utf-8", b"", 0, 1, "test")
)
def test_contains_non_ascii_env_file_non_ascii(
    mock_read: MagicMock, mock_exists: MagicMock
) -> None:
    """Test non-ASCII check when the config file content is not valid UTF-8."""
    assert contains_non_ascii_env()


@patch.dict(os.environ, {"BIJUXCLI_CONFIG": "/path/to/config"})
@patch.object(Path, "exists", return_value=True)
@patch.object(Path, "read_text", side_effect=OSError)
def test_contains_non_ascii_env_file_error(
    mock_read: MagicMock, mock_exists: MagicMock
) -> None:
    """Test non-ASCII check when the config file is unreadable."""
    assert not contains_non_ascii_env()


@patch.dict(os.environ, {"BIJUXCLI_OTHER": "non\u00a9ascii"})
def test_contains_non_ascii_env_other_env() -> None:
    """Test non-ASCII check with another BIJUXCLI-prefixed environment variable."""
    assert contains_non_ascii_env()


@patch.dict(os.environ, {"OTHER": "non\u00a9ascii"})
def test_contains_non_ascii_env_non_bijux() -> None:
    """Test that non-BIJUXCLI-prefixed environment variables are ignored."""
    assert not contains_non_ascii_env()


def test_validate_common_flags_valid_json() -> None:
    """Test common flag validation with 'json' format."""
    assert validate_common_flags("json", "cmd", False) is OutputFormat.JSON


def test_validate_common_flags_valid_yaml() -> None:
    """Test common flag validation with 'YAML' format."""
    assert validate_common_flags("YAML", "cmd", False) is OutputFormat.YAML


def test_validate_common_flags_invalid() -> None:
    """Test common flag validation with an invalid format."""
    with pytest.raises(ExitIntentError) as exc:
        validate_common_flags("invalid", "cmd", False)
    intent = exc.value.intent
    payload = cast(dict[str, Any], intent.payload)
    assert payload["failure"] == "format"
    assert payload["command"] == "cmd"
    assert intent.fmt == OutputFormat.JSON


def test_validate_common_flags_non_ascii(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test common flag validation when non-ASCII environment is detected."""
    monkeypatch.setattr(
        "bijux_cli.cli.core.validation.contains_non_ascii_env", lambda: True
    )
    with pytest.raises(ExitIntentError) as exc:
        validate_common_flags("json", "cmd", False)
    intent = exc.value.intent
    payload = cast(dict[str, Any], intent.payload)
    assert payload["failure"] == "ascii"
    assert payload["command"] == "cmd"
    assert intent.fmt == OutputFormat.JSON


def test_validate_common_flags_quiet() -> None:
    """Test that validation errors are suppressed in quiet mode."""
    assert validate_common_flags("json", "cmd", True) is OutputFormat.JSON


def test_validate_common_flags_include_runtime() -> None:
    """Test that runtime info is included in error payload when requested."""
    with (
        patch("bijux_cli.cli.core.validation.contains_non_ascii_env", lambda: True),
        pytest.raises(ExitIntentError) as exc,
    ):
        validate_common_flags("json", "cmd", False, include_runtime=True)
    intent = exc.value.intent
    payload = cast(dict[str, Any], intent.payload)
    assert payload["python"]
    assert payload["platform"]
    assert payload["timestamp"]


def test_validate_env_file_if_present_no_path() -> None:
    """Test env file validation with no path provided."""
    validate_env_file_if_present("")


def test_validate_env_file_if_present_non_exist(tmp_path: Path) -> None:
    """Test env file validation for a non-existent file."""
    validate_env_file_if_present(str(tmp_path / "non_exist"))


@patch.object(Path, "read_text", side_effect=OSError("read fail"))
def test_validate_env_file_if_present_read_fail(
    mock_read: MagicMock, tmp_path: Path
) -> None:
    """Test env file validation when the file cannot be read."""
    path = tmp_path / "config"
    path.touch()
    with pytest.raises(ValueError, match="Cannot read"):
        validate_env_file_if_present(str(path))


@patch.object(Path, "read_text", return_value="KEY=VALUE\n# comment\nINVALID")
def test_validate_env_file_if_present_invalid_line(
    mock_read: MagicMock, tmp_path: Path
) -> None:
    """Test env file validation with a malformed line."""
    path = tmp_path / "config"
    path.touch()
    with pytest.raises(ValueError, match="Malformed line 3"):
        validate_env_file_if_present(str(path))


@patch.object(
    Path, "read_text", return_value="KEY=VALUE\n# comment\nANOTHER_KEY=value123"
)
def test_validate_env_file_if_present_valid(
    mock_read: MagicMock, tmp_path: Path
) -> None:
    """Test env file validation with a valid file."""
    path = tmp_path / "config"
    path.touch()
    validate_env_file_if_present(str(path))


def test_new_run_command_success(mock_di: types.SimpleNamespace) -> None:
    """Test a successful command execution via new_run_command."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):

        def builder(include: bool) -> dict[str, str]:
            return {"test": "value"}

        with pytest.raises(ExitIntentError) as exc:
            cast(Any, new_run_command)(
                "cmd",
                builder,
                quiet=False,
                fmt=OutputFormat.JSON,
                pretty=True,
                log_level=LogLevel.INFO,
            )
        intent = exc.value.intent
        assert intent.payload == {"test": "value"}
        assert intent.stream == "stdout"
        assert intent.fmt == OutputFormat.JSON


@patch(
    "bijux_cli.cli.core.command.validate_common_flags", return_value=OutputFormat.YAML
)
def test_new_run_command_yaml(
    mock_validate: MagicMock, mock_di: types.SimpleNamespace
) -> None:
    """Test command execution with YAML output format."""

    def builder(include: bool) -> dict[str, str]:
        return {"test": "value"}

    with pytest.raises(ExitIntentError) as exc:
        cast(Any, new_run_command)(
            "cmd",
            builder,
            quiet=False,
            fmt=OutputFormat.JSON,
            pretty=True,
            log_level=LogLevel.INFO,
        )
    intent = exc.value.intent
    assert intent.payload == {"test": "value"}
    assert intent.fmt == OutputFormat.YAML


def test_new_run_command_build_fail(mock_di: types.SimpleNamespace) -> None:
    """Test command execution where the payload builder fails."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):

        def builder(include: bool) -> dict[str, Any]:
            raise ValueError("build fail")

        with pytest.raises(ExitIntentError) as exc:
            cast(Any, new_run_command)(
                "cmd",
                builder,
                quiet=False,
                fmt=OutputFormat.JSON,
                pretty=True,
                log_level=LogLevel.INFO,
            )
        intent = exc.value.intent
        payload = cast(dict[str, Any], intent.payload)
        assert payload["failure"] == "ascii"
        assert payload["error"] == "build fail"


def test_new_run_command_history_skip_quiet(mock_di: types.SimpleNamespace) -> None:
    """Test that history is skipped in quiet mode."""
    with (
        patch(
            "bijux_cli.cli.core.command.current_execution_policy",
            return_value=ExecutionPolicy(
                output_format=OutputFormat.JSON,
                color=ColorMode.AUTO,
                quiet=True,
                log_level=LogLevel.ERROR,
                pretty=True,
                include_runtime=False,
            ),
        ),
        patch(
            "bijux_cli.cli.core.command.validate_common_flags",
            return_value=OutputFormat.JSON,
        ),
    ):

        def builder(include: bool) -> dict[str, Any]:
            return {}

        with pytest.raises(ExitIntentError) as exc:
            new_run_command(
                "cmd",
                builder,
                quiet=True,
                fmt=OutputFormat.JSON,
                pretty=True,
                log_level=LogLevel.INFO,
            )
        with pytest.raises(typer.Exit):
            execute_exit_intent(exc.value.intent)
        assert any(
            call.args[0] == HistoryProtocol for call in mock_di.resolve.call_args_list
        )


def test_new_run_command_history_skip_history_cmd(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test that the history command itself is not recorded in history."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):

        def builder(include: bool) -> dict[str, Any]:
            return {}

        with pytest.raises(ExitIntentError) as exc:
            new_run_command(
                "history",
                builder,
                quiet=False,
                fmt=OutputFormat.JSON,
                pretty=True,
                log_level=LogLevel.INFO,
            )
        with pytest.raises(typer.Exit):
            execute_exit_intent(exc.value.intent)
        assert not any(
            call.args[0] == HistoryProtocol for call in mock_di.resolve.call_args_list
        )


def test_new_run_command_history_success(mock_di: types.SimpleNamespace) -> None:
    """Test successful command recording in history."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):
        mock_hist = MagicMock(spec=HistoryProtocol)

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect

        def builder(include: bool) -> dict[str, Any]:
            return {}

        with pytest.raises(ExitIntentError) as exc:
            new_run_command(
                "cmd",
                builder,
                quiet=False,
                fmt=OutputFormat.JSON,
                pretty=True,
                log_level=LogLevel.INFO,
            )
        with pytest.raises(typer.Exit):
            execute_exit_intent(exc.value.intent)
        mock_hist.add.assert_called_with(
            command="cmd", params=[], success=True, return_code=0, duration_ms=0.0
        )


def test_new_run_command_history_fail(mock_di: types.SimpleNamespace) -> None:
    """Test failed command recording in history."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):
        mock_hist = MagicMock(spec=HistoryProtocol)

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect

        def builder(include: bool) -> dict[str, Any]:
            return {}

        with pytest.raises(ExitIntentError) as exc:
            new_run_command(
                "cmd",
                builder,
                quiet=False,
                fmt=OutputFormat.JSON,
                pretty=True,
                log_level=LogLevel.INFO,
                exit_code=1,
            )
        with pytest.raises(typer.Exit):
            execute_exit_intent(exc.value.intent)
        mock_hist.add.assert_called_with(
            command="cmd", params=[], success=False, return_code=1, duration_ms=0.0
        )


def test_new_run_command_history_permission_error(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of PermissionError when writing history."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = PermissionError("perm error")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(ExitIntentError) as exc:
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    fmt=OutputFormat.JSON,
                    pretty=True,
                    log_level=LogLevel.INFO,
                )
            with pytest.raises(typer.Exit):
                execute_exit_intent(exc.value.intent)
            mock_print.assert_any_call(
                "Permission denied writing history: perm error", file=sys.stderr
            )


def test_new_run_command_history_os_error_perm(mock_di: types.SimpleNamespace) -> None:
    """Test handling of OSError EACCES/EPERM when writing history."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = OSError(13, "perm")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(ExitIntentError) as exc:
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    fmt=OutputFormat.JSON,
                    pretty=True,
                    log_level=LogLevel.INFO,
                )
            with pytest.raises(typer.Exit):
                execute_exit_intent(exc.value.intent)
            mock_print.assert_any_call(
                "Permission denied writing history: [Errno 13] perm", file=sys.stderr
            )


def test_new_run_command_history_os_error_space(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of OSError ENOSPC when writing history."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = OSError(28, "no space")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(ExitIntentError) as exc:
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    fmt=OutputFormat.JSON,
                    pretty=True,
                    log_level=LogLevel.INFO,
                )
            with pytest.raises(typer.Exit):
                execute_exit_intent(exc.value.intent)
            mock_print.assert_any_call(
                "No space left on device while writing history: [Errno 28] no space",
                file=sys.stderr,
            )


def test_new_run_command_history_os_error_other(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of other OSErrors when writing history."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = OSError(5, "io error")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(ExitIntentError) as exc:
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    fmt=OutputFormat.JSON,
                    pretty=True,
                    log_level=LogLevel.INFO,
                )
            with pytest.raises(typer.Exit):
                execute_exit_intent(exc.value.intent)
            mock_print.assert_any_call(
                "Error writing history: [Errno 5] io error", file=sys.stderr
            )


def test_new_run_command_history_exception(mock_di: types.SimpleNamespace) -> None:
    """Test handling of generic exceptions when writing history."""
    with patch(
        "bijux_cli.cli.core.command.validate_common_flags",
        return_value=OutputFormat.JSON,
    ):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = Exception("other error")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(ExitIntentError) as exc:
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    fmt=OutputFormat.JSON,
                    pretty=True,
                    log_level=LogLevel.INFO,
                )
            with pytest.raises(typer.Exit):
                execute_exit_intent(exc.value.intent)
            mock_print.assert_any_call(
                "Error writing history: other error", file=sys.stderr
            )


def test_raise_exit_intent_builds_payload() -> None:
    """Test that raise_exit_intent raises an intent with payload."""
    with pytest.raises(ExitIntentError) as exc:
        raise_exit_intent(
            "boom",
            code=1,
            failure="internal",
            error_type=ErrorType.INTERNAL,
            command="cmd",
            fmt=OutputFormat.JSON,
            quiet=False,
            include_runtime=True,
            log_level=LogLevel.INFO,
        )
    intent = exc.value.intent
    payload = cast(dict[str, Any], intent.payload)
    assert payload["error"] == "boom"
    assert payload["failure"] == "internal"
    assert payload["command"] == "cmd"
    assert payload["python"]
    assert payload["platform"]
    assert intent.stream == "stderr"
