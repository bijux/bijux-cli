# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the utilities module."""

from __future__ import annotations

from collections.abc import Callable
import errno
import json
import os
from pathlib import Path
import sys
import types
from typing import Any, cast
from unittest.mock import ANY, MagicMock, patch

import pytest

from bijux_cli.commands.utilities import (
    ascii_safe,
    contains_non_ascii_env,
    emit_and_exit,
    emit_error_and_exit,
    handle_list_plugins,
    list_installed_plugins,
    new_run_command,
    normalize_format,
    parse_global_flags,
    validate_common_flags,
    validate_env_file_if_present,
)
from bijux_cli.contracts import HistoryProtocol
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import OutputFormat


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
    with patch("bijux_cli.commands.utilities.emit_error_and_exit") as mock_exit:
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
    monkeypatch.setattr(
        "bijux_cli.commands.utilities.contains_non_ascii_env", lambda: True
    )
    with patch("bijux_cli.commands.utilities.emit_error_and_exit") as mock_exit:
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
    with patch("bijux_cli.commands.utilities.emit_error_and_exit") as mock_exit:
        validate_common_flags("json", "cmd", True)
        mock_exit.assert_not_called()


def test_validate_common_flags_include_runtime() -> None:
    """Test that runtime info is included in error payload when requested."""
    with (
        patch("bijux_cli.commands.utilities.contains_non_ascii_env", lambda: True),
        patch("bijux_cli.commands.utilities.emit_error_and_exit") as mock_exit,
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
            "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.utilities.emit_and_exit") as mock_emit_exit,
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
            debug=False,
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


@patch("bijux_cli.commands.utilities.validate_common_flags", return_value="yaml")
def test_new_run_command_yaml(
    mock_validate: MagicMock, mock_di: types.SimpleNamespace
) -> None:
    """Test command execution with YAML output format."""
    with patch("bijux_cli.commands.utilities.emit_and_exit") as mock_emit_exit:

        def builder(include: bool) -> dict[str, str]:
            return {"test": "value"}

        cast(Any, new_run_command)(
            "cmd",
            builder,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
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
            "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.utilities.emit_error_and_exit") as mock_error_exit,
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
            debug=False,
        )
        mock_error_exit.assert_called_with(
            "build fail",
            code=3,
            failure="ascii",
            command="cmd",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_new_run_command_history_skip_quiet(mock_di: types.SimpleNamespace) -> None:
    """Test that history is skipped in quiet mode."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):

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
                debug=False,
            )
        assert not any(
            call.args[0] == HistoryProtocol for call in mock_di.resolve.call_args_list
        )


def test_new_run_command_history_skip_history_cmd(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test that the history command itself is not recorded in history."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):

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
                debug=False,
            )
        assert not any(
            call.args[0] == HistoryProtocol for call in mock_di.resolve.call_args_list
        )


def test_new_run_command_history_success(mock_di: types.SimpleNamespace) -> None:
    """Test successful command recording in history."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):
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
                debug=False,
            )
        mock_hist.add.assert_called_with(
            command="cmd", params=[], success=True, return_code=0, duration_ms=0.0
        )


def test_new_run_command_history_fail(mock_di: types.SimpleNamespace) -> None:
    """Test failed command recording in history."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):
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
                debug=False,
                exit_code=1,
            )
        mock_hist.add.assert_called_with(
            command="cmd", params=[], success=False, return_code=1, duration_ms=0.0
        )


def test_new_run_command_history_permission_error(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of PermissionError when writing history."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):
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
                    debug=False,
                )
            mock_print.assert_any_call(
                "Permission denied writing history: perm error", file=sys.stderr
            )


def test_new_run_command_history_os_error_perm(mock_di: types.SimpleNamespace) -> None:
    """Test handling of OSError EACCES/EPERM when writing history."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):
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
                    debug=False,
                )
            mock_print.assert_any_call(
                "Permission denied writing history: [Errno 13] perm", file=sys.stderr
            )


def test_new_run_command_history_os_error_space(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of OSError ENOSPC when writing history."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):
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
                    debug=False,
                )
            mock_print.assert_any_call(
                "No space left on device while writing history: [Errno 28] no space",
                file=sys.stderr,
            )


def test_new_run_command_history_os_error_other(
    mock_di: types.SimpleNamespace,
) -> None:
    """Test handling of other OSErrors when writing history."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):
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
                    debug=False,
                )
            mock_print.assert_any_call(
                "Error writing history: [Errno 5] io error", file=sys.stderr
            )


def test_new_run_command_history_exception(mock_di: types.SimpleNamespace) -> None:
    """Test handling of generic exceptions when writing history."""
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):
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
                    debug=False,
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
    with patch("json.dumps") as mock_dumps, patch("builtins.print") as mock_print:
        mock_dumps.return_value = '{"key": "value"}\n'
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
    mock_dumps.assert_called_with({"key": "value"}, indent=2, separators=(", ", ": "))
    mock_print.assert_called_with('{"key": "value"}')


def test_emit_and_exit_json_compact() -> None:
    """Test compact JSON output."""
    with patch("json.dumps") as mock_dumps, patch("builtins.print") as mock_print:
        mock_dumps.return_value = '{"key":"value"}\n'
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
    mock_dumps.assert_called_with({"key": "value"}, indent=None, separators=(",", ":"))
    mock_print.assert_called_with('{"key":"value"}')


def test_emit_and_exit_yaml_pretty() -> None:
    """Test pretty-printed YAML output."""
    with patch("yaml.safe_dump") as mock_dump, patch("builtins.print") as mock_print:
        mock_dump.return_value = "key: value\n"
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
    mock_dump.assert_called_with(
        {"key": "value"}, indent=2, sort_keys=False, default_flow_style=None
    )
    mock_print.assert_called_with("key: value")


def test_emit_and_exit_yaml_compact() -> None:
    """Test compact YAML output."""
    with patch("yaml.safe_dump") as mock_dump, patch("builtins.print") as mock_print:
        mock_dump.return_value = "key: value\n"
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
    mock_dump.assert_called_with(
        {"key": "value"}, indent=None, sort_keys=False, default_flow_style=True
    )
    mock_print.assert_called_with("key: value")


def test_emit_and_exit_debug() -> None:
    """Test that diagnostics are printed in debug mode."""
    with (
        patch("json.dumps", return_value='{"key": "value"}\n'),
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
    with patch("json.dumps") as mock_dumps, patch("builtins.print") as mock_print:
        mock_dumps.return_value = '{"error": "test"}\n'
        with pytest.raises(SystemExit):
            emit_error_and_exit("test", 1, "fail", "cmd", "json", False)
    mock_dumps.assert_called_with(ANY)
    mock_print.assert_called_with('{"error": "test"}', file=sys.stderr, flush=True)


def test_emit_error_and_exit_include_runtime() -> None:
    """Test that runtime info is included in error payload when requested."""
    with patch("json.dumps") as mock_dumps, patch("builtins.print"):
        mock_dumps.return_value = '{"error": "test"}\n'
        with pytest.raises(SystemExit):
            emit_error_and_exit("test", 1, "fail", "cmd", "json", False, True)
    mock_dumps.assert_called_with(ANY)
    assert "python" in mock_dumps.call_args[0][0]
    assert "platform" in mock_dumps.call_args[0][0]
    assert "timestamp" in mock_dumps.call_args[0][0]


def test_emit_error_and_exit_extra() -> None:
    """Test that extra data can be added to the error payload."""
    with patch("json.dumps") as mock_dumps, patch("builtins.print"):
        mock_dumps.return_value = '{"error": "test", "extra": "data"}\n'
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
    assert "extra" in mock_dumps.call_args[0][0]


def test_emit_error_and_exit_debug() -> None:
    """Test that a traceback is printed in debug mode."""
    with (
        patch("json.dumps", return_value='{"error": "test"}\n'),
        patch("builtins.print"),
        patch("traceback.print_exc") as mock_tb,
        pytest.raises(SystemExit),
    ):
        emit_error_and_exit("test", 1, "fail", "cmd", "json", False, False, True)
    mock_tb.assert_called_once()


def test_parse_global_flags_empty() -> None:
    """Test parsing global flags with no arguments."""
    sys.argv = ["bijux"]
    flags = parse_global_flags()
    assert flags == {
        "help": False,
        "quiet": False,
        "debug": False,
        "verbose": False,
        "format": "json",
        "pretty": True,
    }
    assert sys.argv == ["bijux"]


def test_parse_global_flags_help() -> None:
    """Test parsing the --help flag."""
    sys.argv = ["bijux", "--help"]
    flags = parse_global_flags()
    assert flags["help"] is True
    assert sys.argv == ["bijux", "--help"]


def test_parse_global_flags_quiet() -> None:
    """Test parsing the --quiet (-q) flag."""
    sys.argv = ["bijux", "-q"]
    flags = parse_global_flags()
    assert flags["quiet"] is True


def test_parse_global_flags_debug() -> None:
    """Test parsing the --debug flag."""
    sys.argv = ["bijux", "--debug"]
    flags = parse_global_flags()
    assert flags["debug"] is True
    assert flags["verbose"] is True
    assert flags["pretty"] is True


def test_parse_global_flags_verbose() -> None:
    """Test parsing the --verbose (-v) flag."""
    sys.argv = ["bijux", "-v"]
    flags = parse_global_flags()
    assert flags["verbose"] is True


def test_parse_global_flags_format() -> None:
    """Test parsing the --format (-f) flag."""
    sys.argv = ["bijux", "-f", "yaml"]
    flags = parse_global_flags()
    assert flags["format"] == "yaml"


def test_parse_global_flags_format_missing() -> None:
    """Test parsing a format flag with a missing argument."""
    sys.argv = ["bijux", "-f"]
    with patch("bijux_cli.commands.utilities.emit_error_and_exit") as mock_exit:
        parse_global_flags()
        mock_exit.assert_called_with(
            ANY,
            code=2,
            failure="missing_argument",
            command="global",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_parse_global_flags_format_invalid_no_help() -> None:
    """Test parsing an invalid format flag when --help is not present."""
    sys.argv = ["bijux", "-f", "invalid"]
    with patch("bijux_cli.commands.utilities.emit_error_and_exit") as mock_exit:
        parse_global_flags()
        mock_exit.assert_called_with(
            ANY,
            code=2,
            failure="invalid_format",
            command="global",
            fmt="invalid",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_parse_global_flags_format_invalid_help() -> None:
    """Test that an invalid format flag is ignored when --help is present."""
    sys.argv = ["bijux", "--help", "-f", "invalid"]
    _ = parse_global_flags()
    assert sys.argv == ["bijux", "--help", "f", "invalid"]


def test_parse_global_flags_pretty() -> None:
    """Test parsing the --pretty flag."""
    sys.argv = ["bijux", "--pretty"]
    flags = parse_global_flags()
    assert flags["pretty"] is True


def test_parse_global_flags_no_pretty() -> None:
    """Test parsing the --no-pretty flag."""
    sys.argv = ["bijux", "--no-pretty"]
    flags = parse_global_flags()
    assert flags["pretty"] is False


def test_parse_global_flags_unknown() -> None:
    """Test that unknown flags are left in sys.argv for later parsing."""
    sys.argv = ["bijux", "--unknown"]
    _ = parse_global_flags()
    assert sys.argv == ["bijux", "--unknown"]


def test_parse_global_flags_unknown_help() -> None:
    """Test that unknown flags are ignored when --help is present."""
    sys.argv = ["bijux", "--help", "--unknown"]
    _ = parse_global_flags()
    assert sys.argv == ["bijux", "--help", "unknown"]


def test_list_installed_plugins_non_exist() -> None:
    """Test listing plugins when the plugins directory does not exist."""
    with patch(
        "bijux_cli.commands.utilities.get_plugins_dir", return_value=Path("/non_exist")
    ):
        assert list_installed_plugins() == []


def test_list_installed_plugins_symlink_loop() -> None:
    """Test that a symlink loop in the plugins directory raises an error."""
    mock_path = MagicMock()
    mock_path.resolve.side_effect = RuntimeError
    with (
        patch("bijux_cli.commands.utilities.get_plugins_dir", return_value=mock_path),
        pytest.raises(RuntimeError, match="Symlink loop"),
    ):
        list_installed_plugins()


def test_list_installed_plugins_not_dir(tmp_path: Path) -> None:
    """Test that an error is raised if the plugins path is a file."""
    file_path = tmp_path / "file"
    file_path.touch()
    with (
        patch("bijux_cli.commands.utilities.get_plugins_dir", return_value=file_path),
        pytest.raises(RuntimeError, match="not a directory"),
    ):
        list_installed_plugins()


def test_list_installed_plugins_invalid_access(tmp_path: Path) -> None:
    """Test that an error is raised if the plugins directory is inaccessible."""
    with (
        patch("bijux_cli.commands.utilities.get_plugins_dir", return_value=tmp_path),
        patch.object(Path, "resolve", side_effect=OSError("access denied")),
        pytest.raises(RuntimeError, match="invalid or inaccessible"),
    ):
        list_installed_plugins()


def test_list_installed_plugins_success(tmp_path: Path) -> None:
    """Test successfully listing valid plugins."""
    plugin1 = tmp_path / "plugin1"
    plugin1.mkdir()
    (plugin1 / "plugin.py").touch()
    plugin2 = tmp_path / "plugin2"
    plugin2.mkdir()
    (plugin2 / "plugin.py").touch()
    invalid = tmp_path / "invalid"
    invalid.mkdir()
    with patch("bijux_cli.commands.utilities.get_plugins_dir", return_value=tmp_path):
        plugins = list_installed_plugins()
        assert plugins == ["plugin1", "plugin2"]


def test_list_installed_plugins_symlink_dir(tmp_path: Path) -> None:
    """Test that symlinked plugin directories are correctly identified."""
    real = tmp_path / "real"
    real.mkdir()
    (real / "plugin.py").touch()
    sym = tmp_path / "sym"
    sym.symlink_to(real)
    with patch("bijux_cli.commands.utilities.get_plugins_dir", return_value=tmp_path):
        plugins = list_installed_plugins()
        assert "sym" in plugins


def test_list_installed_plugins_ignore_errors(tmp_path: Path) -> None:
    """Test that invalid entries in the plugins directory are ignored."""
    plugin = tmp_path / "plugin"
    plugin.mkdir()
    (plugin / "plugin.py").touch()
    invalid = tmp_path / "invalid"
    invalid.touch()
    with patch("bijux_cli.commands.utilities.get_plugins_dir", return_value=tmp_path):
        plugins = list_installed_plugins()
        assert plugins == ["plugin"]


def test_handle_list_plugins_success(
    tmp_path: Path, mock_di: types.SimpleNamespace
) -> None:
    """Test the successful execution of the list-plugins handler."""
    with (
        patch(
            "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
        ),
        patch(
            "bijux_cli.commands.utilities.list_installed_plugins",
            return_value=["p1", "p2"],
        ),
        patch("bijux_cli.commands.utilities.new_run_command") as mock_run,
    ):
        handle_list_plugins("list-plugins", False, False, "json", True, False)
        mock_run.assert_called_with(
            command_name="list-plugins",
            payload_builder=ANY,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
        )
        builder: Callable[[bool], dict[str, Any]] = mock_run.call_args.kwargs[
            "payload_builder"
        ]
        assert builder(False) == {"plugins": ["p1", "p2"]}
        assert "python" in builder(True)


def test_handle_list_plugins_fail(
    tmp_path: Path, mock_di: types.SimpleNamespace
) -> None:
    """Test the failure path of the list-plugins handler."""
    with (
        patch(
            "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
        ),
        patch(
            "bijux_cli.commands.utilities.list_installed_plugins",
            side_effect=RuntimeError("dir error"),
        ),
        patch("bijux_cli.commands.utilities.emit_error_and_exit") as mock_error,
    ):
        handle_list_plugins("list-plugins", False, False, "json", True, False)
        mock_error.assert_called_with(
            "dir error",
            code=1,
            failure="dir_error",
            command="list-plugins",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_parse_global_flags_multiple() -> None:
    """Test parsing multiple global flags at once."""
    sys.argv = [
        "bijux",
        "-q",
        "--debug",
        "-f",
        "yaml",
        "--no-pretty",
        "-v",
        "--unknown",
    ]
    flags = parse_global_flags()
    assert flags == {
        "help": False,
        "quiet": True,
        "debug": True,
        "verbose": True,
        "format": "yaml",
        "pretty": False,
    }
    assert sys.argv == ["bijux", "--unknown"]


def test_all_exported() -> None:
    """Test that the __all__ variable contains all expected utilities."""
    from bijux_cli.commands.utilities import __all__

    assert sorted(__all__) == sorted(
        [
            "ascii_safe",
            "contains_non_ascii_env",
            "emit_and_exit",
            "emit_error_and_exit",
            "handle_list_plugins",
            "list_installed_plugins",
            "new_run_command",
            "normalize_format",
            "parse_global_flags",
            "validate_common_flags",
            "validate_env_file_if_present",
        ]
    )


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
        mock_dumps = stack.enter_context(patch("json.dumps"))
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

    payload = mock_dumps.call_args[0][0]
    assert "failure" not in payload


def test_emit_error_and_exit_no_command() -> None:
    """Test that the 'command' key is omitted from the error payload if None."""
    with (
        patch("json.dumps") as mock_dumps,
        patch("builtins.print"),
        pytest.raises(SystemExit),
    ):
        emit_error_and_exit("test", 1, "fail", None, "json", False)
    payload = mock_dumps.call_args[0][0]
    assert "command" not in payload


def test_emit_error_and_exit_no_fmt() -> None:
    """Test that the 'fmt' key is omitted from the error payload if None."""
    with (
        patch("json.dumps") as mock_dumps,
        patch("builtins.print"),
        pytest.raises(SystemExit),
    ):
        emit_error_and_exit("test", 1, "fail", "cmd", None, False)
    payload = mock_dumps.call_args[0][0]
    assert "fmt" not in payload


def test_emit_error_and_exit_json_dumps_fails() -> None:
    """Test fallback error message when JSON serialization of the error fails."""
    with (
        patch("json.dumps", side_effect=ValueError("fail")),
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
    with patch(
        "bijux_cli.commands.utilities.validate_common_flags", return_value="json"
    ):
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
                    debug=False,
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
