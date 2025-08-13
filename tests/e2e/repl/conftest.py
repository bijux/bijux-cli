# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests' fixture for `bijux repl` command."""

from __future__ import annotations

from pathlib import Path

import pytest


@pytest.fixture
def bijux_env(tmp_path: Path) -> dict[str, str]:
    """Provide an isolated environment for a single test.

    This fixture creates a dictionary of environment variables pointing to a
    unique, temporary config file and enabling the application's test mode.

    Args:
        tmp_path: The pytest `tmp_path` fixture for creating temporary files.

    Returns:
        A dictionary containing environment variables for an isolated test run.
    """
    return {
        "BIJUXCLI_CONFIG": str(tmp_path / ".env"),
        "BIJUXCLI_TEST_MODE": "1",
    }
