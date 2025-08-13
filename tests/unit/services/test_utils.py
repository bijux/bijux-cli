# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services utilities module."""

from __future__ import annotations

from unittest.mock import MagicMock, patch

import pytest

from bijux_cli.core.exceptions import BijuxError
from bijux_cli.services.utils import validate_command


def test_validate_command_empty() -> None:
    """Test that providing an empty command list raises an error."""
    with pytest.raises(BijuxError) as excinfo:
        validate_command([])
    assert "Empty command not allowed" in str(excinfo.value)


@patch("os.getenv")
def test_validate_command_not_allowed(mock_getenv: MagicMock) -> None:
    """Test that a command not in the allowed list is rejected."""
    mock_getenv.return_value = "echo,ls"
    with pytest.raises(BijuxError) as excinfo:
        validate_command(["cat", "file.txt"])
    assert "not in allowed list" in str(excinfo.value)


@patch("os.getenv")
@patch("shutil.which")
def test_validate_command_not_found(
    mock_which: MagicMock, mock_getenv: MagicMock
) -> None:
    """Test that a command not found on the system PATH is rejected."""
    mock_getenv.return_value = "echo,cat"
    mock_which.return_value = None
    with pytest.raises(BijuxError) as excinfo:
        validate_command(["cat", "file.txt"])
    msg = str(excinfo.value)
    assert any(sub in msg for sub in ["not found", "not executable"])


@patch("os.getenv")
@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_disallowed_path(
    mock_basename: MagicMock, mock_which: MagicMock, mock_getenv: MagicMock
) -> None:
    """Test that a command whose resolved path does not match the command name is rejected."""
    mock_getenv.return_value = "cat"
    mock_which.return_value = "/bin/cat2"
    mock_basename.side_effect = lambda x: "cat" if x == "cat" else "cat2"
    with pytest.raises(BijuxError) as excinfo:
        validate_command(["cat", "file.txt"])
    assert "Disallowed command path" in str(excinfo.value)


@pytest.mark.parametrize("unsafe_char", [";", "|", "&", ">", "<", "`", "!"])
@patch("os.getenv")
@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_unsafe_arg(
    mock_basename: MagicMock,
    mock_which: MagicMock,
    mock_getenv: MagicMock,
    unsafe_char: str,
) -> None:
    """Test that command arguments containing unsafe characters are rejected."""
    mock_getenv.return_value = "echo"
    mock_which.return_value = "/bin/echo"
    mock_basename.side_effect = lambda x: "echo"
    with pytest.raises(BijuxError) as excinfo:
        validate_command(["echo", f"test{unsafe_char}"])
    assert "Unsafe argument" in str(excinfo.value)


@patch("os.getenv")
@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_success(
    mock_basename: MagicMock, mock_which: MagicMock, mock_getenv: MagicMock
) -> None:
    """Test that a valid and safe command is successfully validated and resolved."""
    mock_getenv.return_value = "echo"
    mock_which.return_value = "/bin/echo"
    mock_basename.side_effect = lambda x: "echo"
    cmd = ["echo", "hello"]
    result = validate_command(cmd)
    assert result == ["/bin/echo", "hello"]


@patch("os.getenv")
@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_success_full_path(
    mock_basename: MagicMock, mock_which: MagicMock, mock_getenv: MagicMock
) -> None:
    """Test that a command provided with a full path is validated correctly."""
    mock_getenv.return_value = "echo"
    mock_which.return_value = "/bin/echo"
    mock_basename.side_effect = lambda x: "echo"
    cmd = ["/bin/echo", "hello"]
    result = validate_command(cmd)
    assert result == ["/bin/echo", "hello"]


@patch("os.getenv")
def test_validate_command_custom_env(mock_getenv: MagicMock) -> None:
    """Test that a custom allowed commands environment variable is respected."""
    mock_getenv.return_value = "custom_cmd"
    with pytest.raises(BijuxError) as excinfo:
        validate_command(["echo", "test"])
    assert "not in allowed list" in str(excinfo.value)


@patch("os.getenv")
@patch("shutil.which")
@patch("os.path.basename")
def test_validate_command_default_env(
    mock_basename: MagicMock, mock_which: MagicMock, mock_getenv: MagicMock
) -> None:
    """Test that the default allowed commands are used when the env var is not set."""
    mock_getenv.return_value = None
    mock_which.return_value = "/bin/grep"
    mock_basename.side_effect = lambda x: "grep"
    result = validate_command(["grep", "pattern"])
    assert result == ["/bin/grep", "pattern"]
