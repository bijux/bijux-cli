# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines core paths for the Bijux CLI's persistent files.

This module centralizes the default filesystem locations for all user-specific
data, such as configuration, command history, and plugins. This provides a
single source of truth for file paths and simplifies management of persistent
state. All paths are relative to the user's home directory within a `.bijux`
folder.
"""

from __future__ import annotations

from pathlib import Path

BIJUX_HOME = Path.home() / ".bijux"
CONFIG_FILE = BIJUX_HOME / ".env"
HISTORY_FILE = BIJUX_HOME / ".history"
MEMORY_FILE = BIJUX_HOME / ".memory.json"
PLUGINS_DIR = BIJUX_HOME / ".plugins"
