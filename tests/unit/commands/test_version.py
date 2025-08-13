# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the version command."""

from __future__ import annotations

from collections.abc import Generator
import re
from typing import Any
from unittest.mock import ANY, MagicMock, patch

import pytest
from typer.testing import CliRunner

from bijux_cli.__version__ import __version__ as cli_version
from bijux_cli.commands.version import (
    _build_payload,  # pyright: ignore[reportPrivateUsage]
    version_app,
)

runner = CliRunner()


@pytest.fixture
def mock_di() -> MagicMock:
    """Provide a mock dependency injection container."""
    di = MagicMock()

    def _resolve_mock(cls: type[Any]) -> MagicMock:
        return MagicMock(spec=cls)

    di.resolve.side_effect = _resolve_mock
    return di


@pytest.fixture
def mock_di_class(mock_di: MagicMock) -> Generator[MagicMock, None, None]:
    """Patch the DIContainer class and yield the mock."""
    with patch("bijux_cli.commands.version.DIContainer") as mock_class:
        mock_class.current.return_value = mock_di
        yield mock_class


def test_build_payload_default() -> None:
    """Test building the default version payload."""
    payload = _build_payload(False)
    assert payload == {"version": cli_version}


@patch("os.environ.get")
def test_build_payload_env_valid(mock_getenv: MagicMock) -> None:
    """Test building the payload with a valid version from environment variables."""
    mock_getenv.return_value = "1.2.3"
    payload = _build_payload(False)
    assert payload == {"version": "1.2.3"}


@patch("os.environ.get")
def test_build_payload_env_invalid_length_short(mock_getenv: MagicMock) -> None:
    """Test that an empty version from env var raises ValueError."""
    mock_getenv.return_value = ""
    with pytest.raises(ValueError, match="empty or too long"):
        _build_payload(False)


@patch("os.environ.get")
def test_build_payload_env_invalid_length_long(mock_getenv: MagicMock) -> None:
    """Test that a too-long version from env var raises ValueError."""
    mock_getenv.return_value = "a" * 1025
    with pytest.raises(ValueError, match="empty or too long"):
        _build_payload(False)


@patch("os.environ.get")
def test_build_payload_env_non_ascii(mock_getenv: MagicMock) -> None:
    """Test that a non-ASCII version from env var raises ValueError."""
    mock_getenv.return_value = "1.2.3\u00a9"
    with pytest.raises(ValueError, match="non-ASCII"):
        _build_payload(False)


@patch("os.environ.get")
def test_build_payload_env_invalid_semver(mock_getenv: MagicMock) -> None:
    """Test that an invalid semantic version from env var raises ValueError."""
    mock_getenv.return_value = "invalid"
    with pytest.raises(ValueError, match="not valid semantic version"):
        _build_payload(False)


@patch("platform.python_version")
@patch("platform.platform")
@patch("time.time")
def test_build_payload_verbose(
    mock_time: MagicMock, mock_platform: MagicMock, mock_python: MagicMock
) -> None:
    """Test building the verbose version payload."""
    mock_python.return_value = "3.11.0"
    mock_platform.return_value = "Darwin"
    mock_time.return_value = 1234567890.0
    payload = _build_payload(True)
    assert payload["version"] == cli_version
    assert payload["python"] == "3.11.0"
    assert payload["platform"] == "Darwin"
    assert payload["timestamp"] == 1234567890.0


def test_version_callback_format_yaml(mock_di_class: MagicMock) -> None:
    """Test the version command with YAML format."""
    with (
        patch("bijux_cli.commands.version.validate_common_flags") as mock_validate,
        patch("bijux_cli.commands.version.new_run_command") as mock_run,
    ):
        mock_validate.return_value = "yaml"
        result = runner.invoke(version_app, ["--format", "yaml"])
        assert result.exit_code == 0
        mock_validate.assert_called_with("yaml", "version", False)
        mock_run.assert_called_with(
            command_name="version",
            payload_builder=ANY,
            quiet=False,
            verbose=False,
            fmt="yaml",
            pretty=True,
            debug=False,
        )


def test_version_callback_subcommand(mock_di_class: MagicMock) -> None:
    """Test that subcommands can be invoked."""

    @version_app.command()
    def sub() -> None:  # pyright: ignore[reportUnusedFunction]
        """A dummy subcommand."""
        print("sub called")

    result = runner.invoke(version_app, ["sub"])
    assert result.exit_code == 0
    assert "sub called" in result.output


def test_payload_builder_lambda(mock_di_class: MagicMock) -> None:
    """Test that the payload builder lambda works as expected."""
    with (
        patch("bijux_cli.commands.version.validate_common_flags"),
        patch("bijux_cli.commands.version.new_run_command") as mock_run,
    ):
        runner.invoke(version_app)
        builder = mock_run.call_args.kwargs["payload_builder"]
        payload_false = builder(False)
        payload_true = builder(True)
        assert "version" in payload_false
        assert len(payload_false) == 1
        assert "version" in payload_true
        assert "python" in payload_true
        assert "platform" in payload_true
        assert "timestamp" in payload_true


def test_build_payload_ascii_safe_fail() -> None:
    """Test payload build failure when ascii_safe fails on version."""
    with (
        patch(
            "bijux_cli.commands.version.ascii_safe",
            side_effect=ValueError("ascii fail"),
        ),
        pytest.raises(ValueError, match="ascii fail"),
    ):
        _build_payload(False)


def test_build_payload_verbose_ascii_safe_fail_python() -> None:
    """Test payload build failure when ascii_safe fails on python version."""
    with patch("bijux_cli.commands.version.ascii_safe") as mock_ascii:
        mock_ascii.side_effect = [cli_version, ValueError("ascii fail")]
        with pytest.raises(ValueError, match="ascii fail"):
            _build_payload(True)


def test_build_payload_verbose_ascii_safe_fail_platform() -> None:
    """Test payload build failure when ascii_safe fails on platform."""
    with patch("bijux_cli.commands.version.ascii_safe") as mock_ascii:
        mock_ascii.side_effect = [cli_version, "python", ValueError("ascii fail")]
        with pytest.raises(ValueError, match="ascii fail"):
            _build_payload(True)


def test_version_callback_no_subcommand(mock_di_class: MagicMock) -> None:
    """Test the version command default behavior without a subcommand."""
    with (
        patch("bijux_cli.commands.version.validate_common_flags") as mock_validate,
        patch("bijux_cli.commands.version.new_run_command") as mock_run,
    ):
        mock_validate.return_value = "json"
        result = runner.invoke(version_app)
        assert result.exit_code == 0
        mock_validate.assert_called_with("json", "version", False)
        mock_run.assert_called_with(
            command_name="version",
            payload_builder=ANY,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
        )


def test_version_callback_quiet(mock_di_class: MagicMock) -> None:
    """Test the version command with the --quiet flag."""
    with (
        patch("bijux_cli.commands.version.validate_common_flags") as mock_validate,
        patch("bijux_cli.commands.version.new_run_command") as mock_run,
    ):
        mock_validate.return_value = "json"
        result = runner.invoke(version_app, ["--quiet"])
        assert result.exit_code == 0
        mock_validate.assert_called_with("json", "version", True)
        mock_run.assert_called_with(
            command_name="version",
            payload_builder=ANY,
            quiet=True,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
        )


def test_version_callback_verbose(mock_di_class: MagicMock) -> None:
    """Test the version command with the --verbose flag."""
    with (
        patch("bijux_cli.commands.version.validate_common_flags") as mock_validate,
        patch("bijux_cli.commands.version.new_run_command") as mock_run,
    ):
        mock_validate.return_value = "json"
        result = runner.invoke(version_app, ["--verbose"])
        assert result.exit_code == 0
        mock_run.assert_called_with(
            command_name="version",
            payload_builder=ANY,
            quiet=False,
            verbose=True,
            fmt="json",
            pretty=True,
            debug=False,
        )


def test_version_callback_no_pretty(mock_di_class: MagicMock) -> None:
    """Test the version command with the --no-pretty flag."""
    with (
        patch("bijux_cli.commands.version.validate_common_flags") as mock_validate,
        patch("bijux_cli.commands.version.new_run_command") as mock_run,
    ):
        mock_validate.return_value = "json"
        result = runner.invoke(version_app, ["--no-pretty"])
        assert result.exit_code == 0
        mock_run.assert_called_with(
            command_name="version",
            payload_builder=ANY,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            debug=False,
        )


def test_version_callback_debug(mock_di_class: MagicMock) -> None:
    """Test the version command with the --debug flag."""
    with (
        patch("bijux_cli.commands.version.validate_common_flags") as mock_validate,
        patch("bijux_cli.commands.version.new_run_command") as mock_run,
    ):
        mock_validate.return_value = "json"
        result = runner.invoke(version_app, ["--debug"])
        assert result.exit_code == 0
        mock_run.assert_called_with(
            command_name="version",
            payload_builder=ANY,
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=True,
        )


def test_version_callback_help(mock_di_class: MagicMock) -> None:
    """Test that the version command's help message is displayed correctly."""
    result = runner.invoke(version_app, ["--help"])
    assert result.exit_code == 0
    out = result.output
    assert "Show the CLI version." in out
    assert "-q, --quiet" in out
    assert "-v, --verbose" in out
    assert "-f, --format" in out
    assert re.search(r"--pretty\s*/\s*--no-pretty", out)
    assert "-d, --debug" in out
