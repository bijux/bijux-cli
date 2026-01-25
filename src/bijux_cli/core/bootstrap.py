# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Thin wrapper around the bootstrap implementation."""

from __future__ import annotations

from bijux_cli.core.bootstrap_flow import main
from bijux_cli.core.intent import split_command_args

_split_command_args = split_command_args

__all__ = ["main", "split_command_args", "_split_command_args"]
