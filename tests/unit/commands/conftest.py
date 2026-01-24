# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Shared fixtures for command unit tests."""

from __future__ import annotations

import pytest

from bijux_cli.core.precedence import default_execution_policy


@pytest.fixture(autouse=True)
def _default_policy(monkeypatch: pytest.MonkeyPatch) -> None:
    """Provide a default execution policy for CLI helpers."""
    monkeypatch.setattr(
        "bijux_cli.cli.core.output.current_execution_policy",
        lambda: default_execution_policy(),
    )
