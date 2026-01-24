# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Defines default filesystem paths for Bijux CLI data."""

from __future__ import annotations

from pathlib import Path

BIJUX_HOME = Path.home() / ".bijux"
CONFIG_FILE = BIJUX_HOME / ".env"
HISTORY_FILE = BIJUX_HOME / ".history"
MEMORY_FILE = BIJUX_HOME / ".memory.json"
PLUGINS_DIR = BIJUX_HOME / ".plugins"

__all__ = [
    "BIJUX_HOME",
    "CONFIG_FILE",
    "HISTORY_FILE",
    "MEMORY_FILE",
    "PLUGINS_DIR",
]
