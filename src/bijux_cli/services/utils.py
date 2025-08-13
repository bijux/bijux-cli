# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides shared utility functions for the CLI's service layer.

This module contains common helper functions used by various service
implementations. It centralizes logic for tasks like input validation and
security checks to ensure consistency and robustness across the service layer.
"""

from __future__ import annotations

import os
import shutil

from bijux_cli.core.exceptions import BijuxError


def validate_command(cmd: list[str]) -> list[str]:
    """Validates a command and its arguments against a whitelist.

    This security function is designed to prevent shell injection vulnerabilities.
    It performs several checks:
    1.  Verifies the command name against an allowlist defined by the
        `BIJUXCLI_ALLOWED_COMMANDS` environment variable.
    2.  Resolves the command's absolute path to ensure it's on the system PATH.
    3.  Checks arguments for forbidden shell metacharacters.

    Args:
        cmd (list[str]): The command and arguments to validate, as a list of
            strings.

    Returns:
        list[str]: A validated and safe command list, with the command name
            replaced by its absolute path, suitable for use with `subprocess.run`
            where `shell=False`.

    Raises:
        BijuxError: If the command is empty, not on the allowlist, not found
            on the system PATH, or if any argument contains unsafe characters.
    """
    if not cmd:
        raise BijuxError("Empty command not allowed", http_status=403)
    env_val = os.getenv("BIJUXCLI_ALLOWED_COMMANDS")
    allowed_commands = (env_val or "echo,ls,cat,grep").split(",")

    cmd_name = os.path.basename(cmd[0])
    if cmd_name not in allowed_commands:
        raise BijuxError(
            f"Command {cmd_name!r} not in allowed list: {allowed_commands}",
            http_status=403,
        )
    resolved_cmd_path = shutil.which(cmd[0])
    if not resolved_cmd_path:
        raise BijuxError(
            f"Command not found or not executable: {cmd[0]!r}", http_status=403
        )
    if os.path.basename(resolved_cmd_path) != cmd_name:
        raise BijuxError(f"Disallowed command path: {cmd[0]!r}", http_status=403)
    cmd[0] = resolved_cmd_path
    forbidden = set(";|&><`!")
    for arg in cmd[1:]:
        if any(ch in arg for ch in forbidden):
            raise BijuxError(f"Unsafe argument: {arg!r}", http_status=403)
    return cmd


__all__ = ["validate_command"]
