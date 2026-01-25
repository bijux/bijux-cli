# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly-only pytest configuration."""

from __future__ import annotations

import os

import pytest


def pytest_configure(config: pytest.Config) -> None:
    """Configure runtime defaults for nightly stress tests."""
    os.environ.setdefault("BIJUXCLI_TEST_TIMEOUT", "60")


def pytest_collection_modifyitems(
    config: pytest.Config, items: list[pytest.Item]
) -> None:
    """Extend timeouts for nightly stress tests."""
    for item in items:
        if "tests/nightly" in str(item.fspath):
            item.add_marker(pytest.mark.timeout(180))
