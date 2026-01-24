# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the core context module."""

from __future__ import annotations

from unittest.mock import MagicMock

import pytest

from bijux_cli.core.context import Context, _current_context
from bijux_cli.core.di import DIContainer


@pytest.fixture
def mock_di() -> MagicMock:
    """Provide a mock DIContainer."""
    return MagicMock(spec=DIContainer)


@pytest.fixture
def context(mock_di: MagicMock) -> Context:
    """Provide a Context instance initialized with a mock DI container."""
    return Context(mock_di)


def test_set_and_get(context: Context) -> None:
    """Set a value and retrieve it."""
    context.set("key", "value")
    assert context.get("key") == "value"


def test_get_missing_raises(context: Context) -> None:
    """Missing keys raise KeyError."""
    with pytest.raises(KeyError, match="not found"):
        context.get("missing")


def test_clear(context: Context) -> None:
    """Clear removes all values."""
    context.set("key", "value")
    context.clear()
    assert context._data == {}


def test_sync_context_manager(context: Context) -> None:
    """Context manager sets and clears current data."""
    with context:
        assert _current_context.get() == context._data
    assert _current_context.get() is None or _current_context.get() == {}


@pytest.mark.asyncio
async def test_async_context_manager(context: Context) -> None:
    """Async context manager sets and clears current data."""
    async with context:
        assert _current_context.get() == context._data
    assert _current_context.get() is None or _current_context.get() == {}
