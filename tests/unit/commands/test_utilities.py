# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for CLI emit/validation/output helpers."""

from __future__ import annotations

import errno
import json
import os
from pathlib import Path
import sys
import types
from typing import Any, cast
from unittest.mock import ANY, MagicMock, patch

import pytest

from bijux_cli.app.di import DIContainer
from bijux_cli.cli.emit import emit_and_exit, emit_error_and_exit
from bijux_cli.cli.flags import parse_global_flags
from bijux_cli.cli.output import new_run_command
from bijux_cli.cli.validation import (
    ascii_safe,
    contains_non_ascii_env,
    normalize_format,
    validate_common_flags,
    validate_env_file_if_present,
)
from bijux_cli.core.enums import OutputFormat
from bijux_cli.services.history.contracts import HistoryProtocol
from bijux_cli.services.plugins.listing import list_installed_plugins


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
    assert normalize_format(None) == ""


def test_normalize_format_empty() -> None:
    """Test normalize_format with an empty string."""
    assert normalize_format("") == ""


def test_normalize_format_whitespace() -> None:
    """Test normalize_format with leading/trailing whitespace."""
    assert normalize_format(" json ") == "json"


def test_normalize_format_upper() -> None:
    """Test normalize_format with an uppercase string."""
    assert normalize_format("YAML") == "yaml"


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
    assert validate_common_flags("json", "cmd", False) == "json"


def test_validate_common_flags_valid_yaml() -> None:
    """Test common flag validation with 'YAML' format."""
    assert validate_common_flags("YAML", "cmd", False) == "yaml"


def test_validate_common_flags_invalid() -> None:
    """Test common flag validation with an invalid format."""
    with patch("bijux_cli.cli.emit.emit_error_and_exit") as mock_exit:
        validate_common_flags("invalid", "cmd", False)
        mock_exit.assert_called_with(
            "Unsupported format: invalid",
            code=2,
            failure="format",
            command="cmd",
            fmt="invalid",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_validate_common_flags_non_ascii(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test common flag validation when non-ASCII environment is detected."""
    monkeypatch.setattr("bijux_cli.cli.validation.contains_non_ascii_env", lambda: True)
    with patch("bijux_cli.cli.emit.emit_error_and_exit") as mock_exit:
        validate_common_flags("json", "cmd", False)
        mock_exit.assert_called_with(
            "Non-ASCII in configuration or environment",
            code=3,
            failure="ascii",
            command="cmd",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_validate_common_flags_quiet() -> None:
    """Test that validation errors are suppressed in quiet mode."""
    with patch("bijux_cli.cli.emit.emit_error_and_exit") as mock_exit:
        validate_common_flags("json", "cmd", True)
        mock_exit.assert_not_called()


def test_validate_common_flags_include_runtime() -> None:
    """Test that runtime info is included in error payload when requested."""
    with (
        patch("bijux_cli.cli.validation.contains_non_ascii_env", lambda: True),
        patch("bijux_cli.cli.emit.emit_error_and_exit") as mock_exit,
    ):
        validate_common_flags("json", "cmd", False, include_runtime=True)
        mock_exit.assert_called_with(
            ANY,
            code=ANY,
            failure=ANY,
            command=ANY,
            fmt=ANY,
            quiet=ANY,
            include_runtime=True,
            debug=ANY,
        )


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
    with (
        patch(
            "bijux_cli.cli.output.validate_common_flags",
            return_value="json",
        ),
        patch("bijux_cli.cli.output.emit_and_exit") as mock_emit_exit,
    ):

        def builder(include: bool) -> dict[str, str]:
            return {"test": "value"}

        cast(Any, new_run_command)(
            "cmd",
            builder,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )
        mock_emit_exit.assert_called_with(
            payload={"test": "value"},
            fmt=OutputFormat.JSON,
            effective_pretty=True,
            verbose=False,
            debug=False,
            quiet=False,
            command="cmd",
            exit_code=0,
        )


@patch("bijux_cli.cli.output.validate_common_flags", return_value="yaml")
def test_new_run_command_yaml(
    mock_validate: MagicMock, mock_di: types.SimpleNamespace
) -> None:
    """Test command execution with YAML output format."""
    with patch("bijux_cli.cli.output.emit_and_exit") as mock_emit_exit:

        def builder(include: bool) -> dict[str, str]:
            return {"test": "value"}

        cast(Any, new_run_command)(
            "cmd",
            builder,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )
        mock_emit_exit.assert_called_with(
            payload={"test": "value"},
            fmt=OutputFormat.YAML,
            effective_pretty=True,
            verbose=False,
            debug=False,
            quiet=False,
            command="cmd",
            exit_code=0,
        )


def test_new_run_command_build_fail(mock_di: types.SimpleNamespace) -> None:
    """Test command execution where the payload builder fails."""
    with (
        patch(
            "bijux_cli.cli.output.validate_common_flags",
            return_value="json",
        ),
        patch("bijux_cli.cli.output.emit_error_and_exit") as mock_error_exit,
    ):

        def builder(include: bool) -> dict[str, Any]:
            raise ValueError("build fail")

        cast(Any, new_run_command)(
            "cmd",
            builder,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            log_level="info",
        )
        mock_error_exit.assert_called_with(
            "build fail",
            code=3,
            failure="ascii",
            command="cmd",
            fmt=OutputFormat.JSON,
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_new_run_command_history_skip_quiet(mock_di: types.SimpleNamespace) -> None:
    """Test that history is skipped in quiet mode."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):

        def builder(include: bool) -> dict[str, Any]:
            return {}

        with pytest.raises(SystemExit):
            new_run_command(
                "cmd",
                builder,
                quiet=True,
                verbose=False,
                fmt="json",
                pretty=True,
                log_level="info",
            )
        assert not any(
            call.args[0] == HistoryProtocol for call in mock_di.resolve.call_args_list
        )


def test_new_run_command_history_skip_history_cmd(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test that the history command itself is not recorded in history."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):

        def builder(include: bool) -> dict[str, Any]:
            return {}

        with pytest.raises(SystemExit):
            new_run_command(
                "history",
                builder,
                quiet=False,
                verbose=False,
                fmt="json",
                pretty=True,
                log_level="info",
            )
        assert not any(
            call.args[0] == HistoryProtocol for call in mock_di.resolve.call_args_list
        )


def test_new_run_command_history_success(mock_di: types.SimpleNamespace) -> None:
    """Test successful command recording in history."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):
        mock_hist = MagicMock(spec=HistoryProtocol)

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect

        def builder(include: bool) -> dict[str, Any]:
            return {}

        with pytest.raises(SystemExit):
            new_run_command(
                "cmd",
                builder,
                quiet=False,
                verbose=False,
                fmt="json",
                pretty=True,
                log_level="info",
            )
        mock_hist.add.assert_called_with(
            command="cmd", params=[], success=True, return_code=0, duration_ms=0.0
        )


def test_new_run_command_history_fail(mock_di: types.SimpleNamespace) -> None:
    """Test failed command recording in history."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):
        mock_hist = MagicMock(spec=HistoryProtocol)

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect

        def builder(include: bool) -> dict[str, Any]:
            return {}

        with pytest.raises(SystemExit):
            new_run_command(
                "cmd",
                builder,
                quiet=False,
                verbose=False,
                fmt="json",
                pretty=True,
                log_level="info",
                exit_code=1,
            )
        mock_hist.add.assert_called_with(
            command="cmd", params=[], success=False, return_code=1, duration_ms=0.0
        )


def test_new_run_command_history_permission_error(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of PermissionError when writing history."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = PermissionError("perm error")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(SystemExit):
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    verbose=False,
                    fmt="json",
                    pretty=True,
                    log_level="info",
                )
            mock_print.assert_any_call(
                "Permission denied writing history: perm error", file=sys.stderr
            )


def test_new_run_command_history_os_error_perm(mock_di: types.SimpleNamespace) -> None:
    """Test handling of OSError EACCES/EPERM when writing history."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = OSError(13, "perm")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(SystemExit):
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    verbose=False,
                    fmt="json",
                    pretty=True,
                    log_level="info",
                )
            mock_print.assert_any_call(
                "Permission denied writing history: [Errno 13] perm", file=sys.stderr
            )


def test_new_run_command_history_os_error_space(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of OSError ENOSPC when writing history."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = OSError(28, "no space")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(SystemExit):
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    verbose=False,
                    fmt="json",
                    pretty=True,
                    log_level="info",
                )
            mock_print.assert_any_call(
                "No space left on device while writing history: [Errno 28] no space",
                file=sys.stderr,
            )


def test_new_run_command_history_os_error_other(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of other OSErrors when writing history."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = OSError(5, "io error")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(SystemExit):
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    verbose=False,
                    fmt="json",
                    pretty=True,
                    log_level="info",
                )
            mock_print.assert_any_call(
                "Error writing history: [Errno 5] io error", file=sys.stderr
            )


def test_new_run_command_history_exception(mock_di: types.SimpleNamespace) -> None:
    """Test handling of generic exceptions when writing history."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = Exception("other error")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(SystemExit):
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    verbose=False,
                    fmt="json",
                    pretty=True,
                    log_level="info",
                )
            mock_print.assert_any_call(
                "Error writing history: other error", file=sys.stderr
            )


def test_emit_and_exit_quiet() -> None:
    """Test that emit_and_exit produces no output in quiet mode."""
    with pytest.raises(SystemExit) as exc:
        emit_and_exit({}, OutputFormat.JSON, True, False, False, True, "cmd")
    assert exc.value.code == 0


def test_emit_and_exit_json_pretty() -> None:
    """Test pretty-printed JSON output."""
    with (
        patch("bijux_cli.cli.emit.resolve_serializer") as mock_factory,
        patch("builtins.print") as mock_print,
    ):
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = '{"key": "value"}\n'
        mock_factory.return_value = mock_serializer
        with pytest.raises(SystemExit):
            emit_and_exit(
                {"key": "value"},
                OutputFormat.JSON,
                True,
                False,
                False,
                False,
                "cmd",
            )
    mock_serializer.dumps.assert_called_with(
        {"key": "value"}, fmt=OutputFormat.JSON, pretty=True
    )
    mock_print.assert_called_with('{"key": "value"}')


def test_emit_and_exit_json_compact() -> None:
    """Test compact JSON output."""
    with (
        patch("bijux_cli.cli.emit.resolve_serializer") as mock_factory,
        patch("builtins.print") as mock_print,
    ):
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = '{"key":"value"}\n'
        mock_factory.return_value = mock_serializer
        with pytest.raises(SystemExit):
            emit_and_exit(
                {"key": "value"},
                OutputFormat.JSON,
                False,
                False,
                False,
                False,
                "cmd",
            )
    mock_serializer.dumps.assert_called_with(
        {"key": "value"}, fmt=OutputFormat.JSON, pretty=False
    )
    mock_print.assert_called_with('{"key":"value"}')


def test_emit_and_exit_yaml_pretty() -> None:
    """Test pretty-printed YAML output."""
    with (
        patch("bijux_cli.cli.emit.resolve_serializer") as mock_factory,
        patch("builtins.print") as mock_print,
    ):
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = "key: value\n"
        mock_factory.return_value = mock_serializer
        with pytest.raises(SystemExit):
            emit_and_exit(
                {"key": "value"},
                OutputFormat.YAML,
                True,
                False,
                False,
                False,
                "cmd",
            )
    mock_serializer.dumps.assert_called_with(
        {"key": "value"}, fmt=OutputFormat.YAML, pretty=True
    )
    mock_print.assert_called_with("key: value")


def test_emit_and_exit_yaml_compact() -> None:
    """Test compact YAML output."""
    with (
        patch("bijux_cli.cli.emit.resolve_serializer") as mock_factory,
        patch("builtins.print") as mock_print,
    ):
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = "key: value\n"
        mock_factory.return_value = mock_serializer
        with pytest.raises(SystemExit):
            emit_and_exit(
                {"key": "value"},
                OutputFormat.YAML,
                False,
                False,
                False,
                False,
                "cmd",
            )
    mock_serializer.dumps.assert_called_with(
        {"key": "value"}, fmt=OutputFormat.YAML, pretty=False
    )
    mock_print.assert_called_with("key: value")


def test_emit_and_exit_debug() -> None:
    """Test that diagnostics are printed in debug mode."""
    with (
        patch(
            "bijux_cli.cli.emit.resolve_serializer",
            return_value=MagicMock(dumps=MagicMock(return_value='{"key": "value"}\n')),
        ),
        patch("builtins.print") as mock_print,
        pytest.raises(SystemExit),
    ):
        emit_and_exit(
            {"key": "value"}, OutputFormat.JSON, True, False, True, False, "cmd"
        )
    mock_print.assert_any_call("Diagnostics: emitted payload", file=sys.stderr)


def test_emit_error_and_exit_quiet() -> None:
    """Test that error output is suppressed in quiet mode."""
    with pytest.raises(SystemExit) as exc:
        emit_error_and_exit("error", 1, "fail", "cmd", "json", True)
    assert exc.value.code == 1


def test_emit_error_and_exit_json() -> None:
    """Test JSON error output."""
    with (
        patch("bijux_cli.cli.emit.resolve_serializer") as mock_factory,
        patch("builtins.print") as mock_print,
    ):
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = '{"error": "test"}\n'
        mock_factory.return_value = mock_serializer
        with pytest.raises(SystemExit):
            emit_error_and_exit("test", 1, "fail", "cmd", "json", False)
    mock_serializer.dumps.assert_called_with(ANY, fmt="json", pretty=False)
    mock_print.assert_called_with('{"error": "test"}', file=sys.stderr, flush=True)


def test_emit_error_and_exit_include_runtime() -> None:
    """Test that runtime info is included in error payload when requested."""
    with (
        patch("bijux_cli.cli.emit.resolve_serializer") as mock_factory,
        patch("builtins.print"),
    ):
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = '{"error": "test"}\n'
        mock_factory.return_value = mock_serializer
        with pytest.raises(SystemExit):
            emit_error_and_exit("test", 1, "fail", "cmd", "json", False, True)
    mock_serializer.dumps.assert_called_with(ANY, fmt="json", pretty=False)
    assert "python" in mock_serializer.dumps.call_args[0][0]
    assert "platform" in mock_serializer.dumps.call_args[0][0]
    assert "timestamp" in mock_serializer.dumps.call_args[0][0]


def test_emit_error_and_exit_extra() -> None:
    """Test that extra data can be added to the error payload."""
    with (
        patch("bijux_cli.cli.emit.resolve_serializer") as mock_factory,
        patch("builtins.print"),
    ):
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = '{"error": "test", "extra": "data"}\n'
        mock_factory.return_value = mock_serializer
        with pytest.raises(SystemExit):
            emit_error_and_exit(
                "test",
                1,
                "fail",
                "cmd",
                "json",
                False,
                False,
                False,
                {"extra": "data"},
            )
    assert "extra" in mock_serializer.dumps.call_args[0][0]


def test_emit_error_and_exit_debug() -> None:
    """Test that a traceback is printed in debug mode."""
    with (
        patch(
            "bijux_cli.cli.emit.resolve_serializer",
            return_value=MagicMock(dumps=MagicMock(return_value='{"error": "test"}\n')),
        ),
        patch("builtins.print"),
        patch("traceback.print_exc") as mock_tb,
        pytest.raises(SystemExit),
    ):
        emit_error_and_exit("test", 1, "fail", "cmd", "json", False, False, True)
    mock_tb.assert_called_once()


def test_parse_global_flags_empty() -> None:
    """Parse global flags with no arguments."""
    flags, retained, errors = parse_global_flags([])
    assert flags["help"] is False
    assert flags["quiet"] is False
    assert flags["verbose"] is False
    assert flags["format"] == "json"
    assert flags["pretty"] is True
    assert retained == []
    assert errors == []


def test_parse_global_flags_help() -> None:
    """Parse the --help flag."""
    flags, retained, errors = parse_global_flags(["--help"])
    assert flags["help"] is True
    assert retained == ["--help"]
    assert errors == []


def test_parse_global_flags_quiet() -> None:
    """Parse the --quiet (-q) flag."""
    flags, retained, errors = parse_global_flags(["-q"])
    assert flags["quiet"] is True
    assert retained == []
    assert errors == []


def test_parse_global_flags_debug() -> None:
    """Parse the --debug flag as a raw flag."""
    flags, retained, errors = parse_global_flags(["--debug"])
    assert flags["debug"] is True
    assert retained == []
    assert errors == []


def test_parse_global_flags_verbose() -> None:
    """Parse the --verbose (-v) flag."""
    flags, retained, errors = parse_global_flags(["-v"])
    assert flags["verbose"] is True
    assert retained == []
    assert errors == []


def test_parse_global_flags_format() -> None:
    """Parse the --format (-f) flag."""
    flags, retained, errors = parse_global_flags(["-f", "yaml"])
    assert flags["format"] == "yaml"
    assert retained == []
    assert errors == []


def test_parse_global_flags_format_missing() -> None:
    """Parse a format flag with a missing argument."""
    flags, retained, errors = parse_global_flags(["-f"])
    assert flags["format"] == "json"
    assert retained == []
    assert errors[0]["failure"] == "missing_argument"


def test_parse_global_flags_format_invalid_help() -> None:
    """Parse invalid format when --help is present."""
    flags, retained, errors = parse_global_flags(["--help", "-f", "invalid"])
    assert flags["help"] is True
    assert retained == ["--help", "f", "invalid"]
    assert errors == []


def test_parse_global_flags_pretty() -> None:
    """Parse the --pretty flag."""
    flags, retained, errors = parse_global_flags(["--pretty"])
    assert flags["pretty"] is True
    assert retained == []
    assert errors == []


def test_parse_global_flags_no_pretty() -> None:
    """Parse the --no-pretty flag."""
    flags, retained, errors = parse_global_flags(["--no-pretty"])
    assert flags["pretty"] is False
    assert retained == []
    assert errors == []


def test_parse_global_flags_unknown() -> None:
    """Unknown flags are retained."""
    flags, retained, errors = parse_global_flags(["--unknown"])
    assert flags["help"] is False
    assert retained == ["--unknown"]
    assert errors == []


def test_parse_global_flags_unknown_help() -> None:
    """Unknown flags are retained with help."""
    flags, retained, errors = parse_global_flags(["--help", "--unknown"])
    assert flags["help"] is True
    assert retained == ["--help", "unknown"]
    assert errors == []


def test_list_installed_plugins_delegates() -> None:
    """Test that plugin listing delegates to metadata."""
    with patch(
        "bijux_cli.services.plugins.listing.list_plugins",
        return_value=[{"name": "p1"}, {"name": "p2"}],
    ):
        assert list_installed_plugins() == [{"name": "p1"}, {"name": "p2"}]


def test_parse_global_flags_multiple() -> None:
    """Parse multiple global flags including --debug."""
    flags, retained, errors = parse_global_flags(
        [
            "-q",
            "--debug",
            "-f",
            "yaml",
            "--no-pretty",
            "-v",
            "--unknown",
        ]
    )
    assert flags["quiet"] is True
    assert flags["debug"] is True
    assert flags["format"] == "yaml"
    assert flags["pretty"] is False
    assert flags["verbose"] is True
    assert retained == ["--unknown"]
    assert errors == []


@patch.dict(os.environ, {"BIJUXCLI_CONFIG": "/path/to/config"})
@patch.object(Path, "exists", return_value=True)
@patch.object(Path, "read_text", return_value="ASCII_OK")
def test_contains_non_ascii_env_file_ascii_ok(
    mock_read: MagicMock, mock_exists: MagicMock
) -> None:
    """Test non-ASCII check with a config file containing only ASCII."""
    assert not contains_non_ascii_env()


def test_emit_error_and_exit_no_failure() -> None:
    """Test that the 'failure' key is omitted from the error payload if None."""
    from contextlib import ExitStack

    with ExitStack() as stack:
        mock_serializer = MagicMock()
        mock_serializer.dumps.return_value = '{"error": "test"}\n'
        stack.enter_context(
            patch(
                "bijux_cli.cli.emit.resolve_serializer",
                return_value=mock_serializer,
            )
        )
        stack.enter_context(patch("builtins.print"))

        failure: Any = None
        with pytest.raises(SystemExit):
            emit_error_and_exit(
                "test",
                1,
                failure,
                "cmd",
                "json",
                False,
            )

    payload = mock_serializer.dumps.call_args[0][0]
    assert "failure" not in payload


def test_emit_error_and_exit_no_command() -> None:
    """Test that the 'command' key is omitted from the error payload if None."""
    with (
        patch(
            "bijux_cli.cli.emit.resolve_serializer",
            return_value=MagicMock(dumps=MagicMock(return_value='{"error":"test"}\n')),
        ) as mock_factory,
        patch("builtins.print"),
        pytest.raises(SystemExit),
    ):
        emit_error_and_exit("test", 1, "fail", None, "json", False)
    payload = mock_factory.return_value.dumps.call_args[0][0]
    assert "command" not in payload


def test_emit_error_and_exit_no_fmt() -> None:
    """Test that the 'fmt' key is omitted from the error payload if None."""
    with (
        patch(
            "bijux_cli.cli.emit.resolve_serializer",
            return_value=MagicMock(dumps=MagicMock(return_value='{"error":"test"}\n')),
        ) as mock_factory,
        patch("builtins.print"),
        pytest.raises(SystemExit),
    ):
        emit_error_and_exit("test", 1, "fail", "cmd", None, False)
    payload = mock_factory.return_value.dumps.call_args[0][0]
    assert "fmt" not in payload


def test_emit_error_and_exit_json_dumps_fails() -> None:
    """Test fallback error message when JSON serialization of the error fails."""
    with (
        patch(
            "bijux_cli.cli.emit.resolve_serializer",
            return_value=MagicMock(dumps=MagicMock(side_effect=ValueError("fail"))),
        ),
        patch("builtins.print") as mock_print,
        pytest.raises(SystemExit),
    ):
        emit_error_and_exit("test", 1, "fail", "cmd", "json", False)
    mock_print.assert_any_call(
        '{"error": "Unserializable error"}', file=sys.stderr, flush=True
    )


@patch.dict(os.environ, {"BIJUXCLI_CONFIG": "safe_ascii_config.env"})
@patch.object(Path, "exists", return_value=True)
@patch.object(Path, "read_text", return_value="ASCII_ONLY_CONTENT")
def test_contains_non_ascii_env_file_ascii_happy(
    mock_read: MagicMock, mock_exists: MagicMock
) -> None:
    """Test non-ASCII check happy path with an all-ASCII config file."""
    assert not contains_non_ascii_env()


def test_emit_and_exit_history_permission_denied(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test history recording failure due to EACCES permission error."""
    with patch("bijux_cli.cli.output.validate_common_flags", return_value="json"):
        mock_hist = MagicMock()
        mock_hist.add.side_effect = OSError(errno.EACCES, "denied")

        def side_effect(cls: type) -> MagicMock:
            return mock_hist if cls == HistoryProtocol else MagicMock()

        mock_di.resolve.side_effect = side_effect
        with patch("builtins.print") as mock_print:

            def builder(include: bool) -> dict[str, Any]:
                return {}

            with pytest.raises(SystemExit):
                new_run_command(
                    "cmd",
                    builder,
                    quiet=False,
                    verbose=False,
                    fmt="json",
                    pretty=True,
                    log_level="info",
                )
            mock_print.assert_any_call(
                "Permission denied writing history: [Errno 13] denied", file=sys.stderr
            )


def test_contains_non_ascii_env_skips_nonexistent_config(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a nonexistent config file path is safely skipped."""
    nonexistent = "nonexistent_config_999999.env"
    monkeypatch.setenv("BIJUXCLI_CONFIG", nonexistent)
    assert not Path(nonexistent).exists()
    assert not contains_non_ascii_env()


def test_emit_and_exit_plain_oserror_eacces_hits_oerror_branch(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test OSError EACCES handling during history recording."""

    class FakeHistory:
        """History that raises EACCES."""

        def add(self, *args: Any, **kwargs: Any) -> None:
            exc = OSError("no write")
            exc.errno = errno.EACCES
            raise exc

    class FakeContainer:
        """Container resolving FakeHistory."""

        def resolve(self, proto: Any) -> Any:
            if proto is HistoryProtocol:
                return FakeHistory()
            raise RuntimeError("unexpected resolve")

    monkeypatch.setattr(
        DIContainer, "current", classmethod(lambda cls: FakeContainer())
    )
    payload = {"result": "ok"}
    with pytest.raises(SystemExit) as se:
        emit_and_exit(
            payload=payload,
            fmt=OutputFormat.JSON,
            effective_pretty=False,
            verbose=False,
            debug=False,
            quiet=False,
            command="mycmd",
            exit_code=0,
        )
    assert se.value.code == 0
    captured = capsys.readouterr()
    assert "Permission denied writing history: no write" in captured.err
    assert json.loads(captured.out) == payload


def test_emit_and_exit_plain_oserror_eperm_hits_oerror_branch(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """Test OSError EPERM handling during history recording."""

    class FakeHistory:
        """A fake history implementation that raises OSError."""

        def add(self, *args: Any, **kwargs: Any) -> None:
            """Raise OSError with EPERM."""
            exc = OSError("op not permitted")
            exc.errno = errno.EPERM
            raise exc

    class FakeContainer:
        """A fake DI container that resolves the fake history."""

        def resolve(self, proto: Any) -> Any:
            """Resolve the FakeHistory protocol."""
            if proto is HistoryProtocol:
                return FakeHistory()
            raise RuntimeError("unexpected resolve")

    monkeypatch.setattr(
        DIContainer, "current", classmethod(lambda cls: FakeContainer())
    )
    payload = {"result": "ok"}
    with pytest.raises(SystemExit) as se:
        emit_and_exit(
            payload=payload,
            fmt=OutputFormat.JSON,
            effective_pretty=False,
            verbose=False,
            debug=False,
            quiet=False,
            command="mycmd",
            exit_code=0,
        )
    assert se.value.code == 0
    captured = capsys.readouterr()
    assert "Permission denied writing history: op not permitted" in captured.err
    assert json.loads(captured.out) == payload
