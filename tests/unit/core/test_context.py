# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the core context module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

from unittest.mock import MagicMock

import pytest

from bijux_cli.contracts import ObservabilityProtocol
from bijux_cli.core.context import Context, _current_context
from bijux_cli.core.di import DIContainer


@pytest.fixture
def mock_di() -> MagicMock:
    """Provide a mock DIContainer."""
    di = MagicMock(spec=DIContainer)
    di.resolve.return_value = MagicMock(spec=ObservabilityProtocol)
    return di


@pytest.fixture
def context(mock_di: MagicMock) -> Context:
    """Provide a Context instance initialized with a mock DI container."""
    return Context(mock_di)


def test_init_verbose(mock_di: MagicMock, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that context initialization logs when verbose mode is enabled."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")
    mock_log = mock_di.resolve.return_value
    Context(mock_di)
    mock_log.log.assert_called_with("debug", "Context initialized", extra={})


def test_init_no_verbose(mock_di: MagicMock, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that context initialization does not log when verbose mode is disabled."""
    monkeypatch.delenv("VERBOSE_DI", raising=False)
    mock_log = mock_di.resolve.return_value
    Context(mock_di)
    mock_log.log.assert_not_called()


def test_init_test_mode(mock_di: MagicMock, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that context initialization does not log when in test mode."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "1")
    mock_log = mock_di.resolve.return_value
    Context(mock_di)
    mock_log.log.assert_not_called()


def test_set_verbose(context: Context, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that setting a context value logs when verbose mode is enabled."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")
    context.set("key", "value")
    assert context._data["key"] == "value"
    context._log.log.assert_called_with(  # type: ignore[attr-defined]
        "debug", "Context set", extra={"key": "key", "value": "value"}
    )


def test_set_no_verbose(context: Context, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that setting a context value does not log when verbose mode is disabled."""
    monkeypatch.delenv("VERBOSE_DI", raising=False)
    context.set("key", "value")
    assert context._data["key"] == "value"
    context._log.log.assert_not_called()  # type: ignore[attr-defined]


def test_get_success_verbose(context: Context, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that getting a context value logs when verbose mode is enabled."""
    context._data["key"] = "value"
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")
    assert context.get("key") == "value"
    context._log.log.assert_called_with(  # type: ignore[attr-defined]
        "debug", "Context get", extra={"key": "key", "value": "value"}
    )


def test_get_success_no_verbose(
    context: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that getting a context value does not log when verbose mode is disabled."""
    context._data["key"] = "value"
    monkeypatch.delenv("VERBOSE_DI", raising=False)
    assert context.get("key") == "value"
    context._log.log.assert_not_called()  # type: ignore[attr-defined]


def test_get_fail_verbose(context: Context, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that failing to get a value logs when verbose mode is enabled."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")
    with pytest.raises(KeyError, match="not found"):
        context.get("missing")
    context._log.log.assert_called_with(  # type: ignore[attr-defined]
        "warning", "Context key not found", extra={"key": "missing"}
    )


def test_get_fail_no_verbose(context: Context, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that failing to get a value does not log when verbose mode is disabled."""
    monkeypatch.delenv("VERBOSE_DI", raising=False)
    with pytest.raises(KeyError, match="not found"):
        context.get("missing")
    context._log.log.assert_not_called()  # type: ignore[attr-defined]


def test_clear_verbose(context: Context, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that clearing the context logs when verbose mode is enabled."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")
    context.clear()
    assert context._data == {}
    context._log.log.assert_called_with(  # type: ignore[attr-defined]
        "debug", "Context cleared", extra={}
    )


def test_clear_no_verbose(context: Context, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that clearing the context does not log when verbose mode is disabled."""
    monkeypatch.delenv("VERBOSE_DI", raising=False)
    context.clear()
    context._log.log.assert_not_called()  # type: ignore[attr-defined]


def test_enter_exit_verbose(context: Context, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that entering and exiting a context logs when verbose mode is enabled."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")
    with context:
        assert _current_context.get() is context._data
    context._log.log.assert_any_call(  # type: ignore[attr-defined]
        "debug", "Context entered", extra={}
    )
    context._log.log.assert_any_call(  # type: ignore[attr-defined]
        "debug", "Context exited", extra={}
    )
    assert _current_context.get() is None


def test_enter_exit_no_verbose(
    context: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that entering and exiting a context does not log when verbose is disabled."""
    monkeypatch.delenv("VERBOSE_DI", raising=False)
    with context:
        assert _current_context.get() is context._data
    context._log.log.assert_not_called()  # type: ignore[attr-defined]
    assert _current_context.get() is None


def test_enter_exit_exception(context: Context) -> None:
    """Test that the context is correctly reset after an exception."""
    with pytest.raises(ValueError, match="test"), context:
        raise ValueError("test")
    assert _current_context.get() is None


@pytest.mark.asyncio
async def test_aenter_aexit_verbose(
    context: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that entering and exiting an async context logs when verbose is enabled."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")
    async with context:
        assert _current_context.get() is context._data
    context._log.log.assert_any_call(  # type: ignore[attr-defined]
        "debug", "Async context entered", extra={}
    )
    context._log.log.assert_any_call(  # type: ignore[attr-defined]
        "debug", "Async context exited", extra={}
    )
    assert _current_context.get() is None


@pytest.mark.asyncio
async def test_aenter_aexit_no_verbose(
    context: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that async enter/exit does not log when verbose is disabled."""
    monkeypatch.delenv("VERBOSE_DI", raising=False)
    async with context:
        assert _current_context.get() is context._data
    context._log.log.assert_not_called()  # type: ignore[attr-defined]
    assert _current_context.get() is None


@pytest.mark.asyncio
async def test_aenter_aexit_exception(context: Context) -> None:
    """Test that the async context is correctly reset after an exception."""
    with pytest.raises(ValueError, match="test"):
        async with context:
            raise ValueError("test")
    assert _current_context.get() is None


def test_current_data_empty() -> None:
    """Test that current_data returns a new empty dict if no context is set."""
    _current_context.set(None)
    data = Context.current_data()
    assert data == {}
    assert _current_context.get() is data


def test_current_data_existing() -> None:
    """Test that current_data returns the existing context data if set."""
    existing = {"existing": "data"}
    _current_context.set(existing)
    data = Context.current_data()
    assert data is existing


def test_set_current_data() -> None:
    """Test that set_current_data correctly updates the context variable."""
    new_data = {"new": "data"}
    Context.set_current_data(new_data)
    assert _current_context.get() is new_data


def test_use_context() -> None:
    """Test the 'use_context' class method context manager."""
    original = _current_context.get()
    temp_data = {"temp": "data"}
    with Context.use_context(temp_data):
        assert _current_context.get() is temp_data
    assert _current_context.get() is original


def test_use_context_exception() -> None:
    """Test that 'use_context' correctly restores the context after an exception."""
    original = _current_context.get()
    temp_data = {"temp": "data"}
    with pytest.raises(ValueError, match="test"), Context.use_context(temp_data):
        raise ValueError("test")
    assert _current_context.get() is original


def test_exit_without_token_noop_and_logs(
    context: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that __exit__ without a token does not reset the context but still logs."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")

    sentinel = {"keep": "me"}
    tok = _current_context.set(sentinel)
    try:
        assert context._token is None
        context.__exit__(None, None, None)
        assert _current_context.get() is sentinel
        context._log.log.assert_called_with(  # type: ignore[attr-defined]
            "debug", "Context exited", extra={}
        )
    finally:
        _current_context.reset(tok)


@pytest.mark.asyncio
async def test_aexit_without_token_noop_and_logs(
    context: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that __aexit__ without a token does not reset the context but still logs."""
    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.setenv("BIJUXCLI_TEST_MODE", "")

    sentinel = {"keep": "me-too"}
    tok = _current_context.set(sentinel)
    try:
        assert context._token is None
        await context.__aexit__(None, None, None)
        assert _current_context.get() is sentinel
        context._log.log.assert_called_with(  # type: ignore[attr-defined]
            "debug", "Async context exited", extra={}
        )
    finally:
        _current_context.reset(tok)
