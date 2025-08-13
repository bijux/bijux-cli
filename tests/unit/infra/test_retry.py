# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra retry module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

import asyncio
from collections.abc import Coroutine, Generator
from contextlib import suppress
from types import TracebackType
from typing import Any
from unittest.mock import MagicMock

import pytest

from bijux_cli.core.exceptions import BijuxError
import bijux_cli.infra.retry as mod
from bijux_cli.infra.retry import (
    ExponentialBackoffRetryPolicy,
    TimeoutRetryPolicy,
    _backoff_loop,
    _close_awaitable,
    _try_asyncio_timeout,
)


class AsyncCtx:
    """A mock async context manager."""

    async def __aenter__(self) -> AsyncCtx:
        """Enter the async context."""
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> bool:
        """Exit the async context."""
        return False


class AwaitableWithClose:
    """An awaitable object that also has a close() method."""

    def __init__(self) -> None:
        """Initialize the awaitable."""
        self.closed = False

    def __await__(self) -> Generator[None, None, None]:
        """Make the object awaitable."""
        yield

    def close(self) -> None:
        """Simulate a close method that records its call."""
        self.closed = True
        raise RuntimeError("boom")


@pytest.fixture
def tel() -> MagicMock:
    """Provide a mock telemetry object."""
    return MagicMock()


@pytest.fixture(autouse=True)
def patch_sleep(monkeypatch: pytest.MonkeyPatch) -> list[float]:
    """Patch asyncio.sleep to avoid actual delays and record call durations."""
    calls: list[float] = []

    async def fake_sleep(d: float) -> None:
        calls.append(d)

    monkeypatch.setattr(asyncio, "sleep", fake_sleep)
    return calls


def test_close_awaitable_calls_and_suppresses() -> None:
    """Test that _close_awaitable calls close() and suppresses any resulting error."""
    obj = AwaitableWithClose()
    _close_awaitable(obj)
    assert obj.closed is True


def test_close_awaitable_noop_without_close() -> None:
    """Test that _close_awaitable does nothing for objects without a close() method."""

    class NoClose:
        pass

    _close_awaitable(NoClose())


def test_try_asyncio_timeout_missing(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that _try_asyncio_timeout returns None if asyncio.timeout is missing."""
    monkeypatch.setattr(asyncio, "timeout", None, raising=False)
    assert _try_asyncio_timeout(0.1) is None


def test_try_asyncio_timeout_not_callable(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that _try_asyncio_timeout returns None if asyncio.timeout is not callable."""
    monkeypatch.setattr(asyncio, "timeout", 123, raising=False)
    assert _try_asyncio_timeout(0.1) is None


def test_try_asyncio_timeout_mock_module_filtered(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a mocked asyncio.timeout from unittest.mock is ignored."""

    def fake_timeout(_: float) -> AsyncCtx:
        return AsyncCtx()

    fake_timeout.__module__ = "unittest.mock"
    monkeypatch.setattr(asyncio, "timeout", fake_timeout, raising=False)
    assert _try_asyncio_timeout(0.1) is None


def test_try_asyncio_timeout_callable_raises(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an exception from asyncio.timeout is caught and returns None."""

    def bad_timeout(_: float) -> None:
        raise TypeError("nope")

    monkeypatch.setattr(asyncio, "timeout", bad_timeout, raising=False)
    assert _try_asyncio_timeout(0.1) is None


def test_try_asyncio_timeout_returns_awaitable(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that an awaitable (non-context manager) from asyncio.timeout is closed."""
    aw = AwaitableWithClose()

    def returns_awaitable(_: float) -> AwaitableWithClose:
        return aw

    monkeypatch.setattr(asyncio, "timeout", returns_awaitable, raising=False)
    assert _try_asyncio_timeout(0.1) is None
    assert aw.closed is True


def test_try_asyncio_timeout_valid_context(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a valid async context manager from asyncio.timeout is returned."""
    monkeypatch.setattr(asyncio, "timeout", lambda s: AsyncCtx(), raising=False)
    ctx = _try_asyncio_timeout(0.2)
    assert ctx is not None
    assert hasattr(ctx, "__aenter__")
    assert hasattr(ctx, "__aexit__")


@pytest.mark.asyncio
async def test_timeout_policy_success_with_ctx(
    monkeypatch: pytest.MonkeyPatch, tel: MagicMock
) -> None:
    """Test the timeout policy's success path using an asyncio.timeout context."""
    monkeypatch.setattr(mod, "_try_asyncio_timeout", lambda s: AsyncCtx())

    async def supplier() -> str:
        return "ok"

    pol = TimeoutRetryPolicy(tel)
    result = await pol.run(supplier, seconds=0.5)
    assert result == "ok"
    tel.event.assert_called_with("retry_timeout_success", {"seconds": 0.5})


@pytest.mark.asyncio
async def test_timeout_policy_success_fallback_wait_for(
    monkeypatch: pytest.MonkeyPatch, tel: MagicMock
) -> None:
    """Test the timeout policy's success path using the asyncio.wait_for fallback."""
    monkeypatch.setattr(mod, "_try_asyncio_timeout", lambda s: None)

    captured: dict[str, Any] = {}

    async def supplier() -> int:
        captured["ran"] = True
        return 42

    async def fake_wait_for(coro: Coroutine[Any, Any, Any], timeout: float) -> Any:
        res = await coro
        captured["timeout"] = timeout
        return res

    monkeypatch.setattr(asyncio, "wait_for", fake_wait_for)
    pol = TimeoutRetryPolicy(tel)
    out = await pol.run(supplier, seconds=1.23)
    assert out == 42
    assert captured["ran"] is True
    assert captured["timeout"] == 1.23
    tel.event.assert_called_with("retry_timeout_success", {"seconds": 1.23})


@pytest.mark.asyncio
async def test_timeout_policy_failure_with_ctx(
    monkeypatch: pytest.MonkeyPatch, tel: MagicMock
) -> None:
    """Test the timeout policy's failure path."""
    monkeypatch.setattr(mod, "_try_asyncio_timeout", lambda s: AsyncCtx())

    async def supplier() -> None:
        raise TimeoutError("late")

    pol = TimeoutRetryPolicy(tel)
    with pytest.raises(BijuxError, match="Operation timed out after 0.5s"):
        await pol.run(supplier, seconds=0.5)
    tel.event.assert_called_with(
        "retry_timeout_failed", {"seconds": 0.5, "error": "late"}
    )


def test_timeout_policy_invalid_seconds(tel: MagicMock) -> None:
    """Test that the timeout policy rejects non-positive second values."""
    pol = TimeoutRetryPolicy(tel)
    with pytest.raises(ValueError, match="seconds must be > 0"):
        asyncio.run(pol.run(lambda: asyncio.sleep(0), seconds=0))


def test_timeout_policy_reset_emits(tel: MagicMock) -> None:
    """Test that the timeout policy's reset method emits a telemetry event."""
    pol = TimeoutRetryPolicy(tel)
    pol.reset()
    tel.event.assert_called_with("retry_reset", {})


@pytest.mark.asyncio
async def test_backoff_loop_success_first_try(tel: MagicMock) -> None:
    """Test that the backoff loop succeeds on the first attempt."""

    async def supplier() -> str:
        return "done"

    out = await _backoff_loop(
        supplier,
        retries=1,
        delay=1.0,
        backoff=2.0,
        jitter=0.0,
        retry_on=(Exception,),
        telemetry=tel,
    )
    assert out == "done"
    tel.event.assert_called_with("retry_async_success", {"retries": 0})


@pytest.mark.asyncio
async def test_backoff_loop_retries_then_success(
    patch_sleep: list[float], tel: MagicMock
) -> None:
    """Test that the backoff loop retries on failure and then succeeds."""
    attempts = {"n": 0}

    async def supplier() -> str:
        attempts["n"] += 1
        if attempts["n"] < 3:
            raise RuntimeError("no")
        return "ok"

    out = await _backoff_loop(
        supplier,
        retries=3,
        delay=1.5,
        backoff=2.0,
        jitter=0.0,
        retry_on=(RuntimeError,),
        telemetry=tel,
    )
    assert out == "ok"
    assert patch_sleep == [1.5, 3.0]
    tel.event.assert_called_with("retry_async_success", {"retries": 2})


@pytest.mark.asyncio
async def test_backoff_loop_exhausts_and_fails(tel: MagicMock) -> None:
    """Test that the backoff loop fails after exhausting all retries."""

    async def supplier() -> None:
        raise ValueError("bad")

    with pytest.raises(ValueError, match="bad"):
        await _backoff_loop(
            supplier,
            retries=2,
            delay=0.1,
            backoff=2.0,
            jitter=0.0,
            retry_on=(ValueError,),
            telemetry=tel,
        )
    tel.event.assert_any_call("retry_async_failed", {"retries": 2, "error": "bad"})


def test_backoff_policy_invalid_seconds(tel: MagicMock) -> None:
    """Test that the backoff policy rejects non-positive second values."""
    pol = ExponentialBackoffRetryPolicy(tel)
    with pytest.raises(ValueError, match="seconds must be > 0"):
        asyncio.run(pol.run(lambda: asyncio.sleep(0), seconds=0))


@pytest.mark.asyncio
async def test_backoff_policy_success_with_ctx(
    monkeypatch: pytest.MonkeyPatch, tel: MagicMock, patch_sleep: list[float]
) -> None:
    """Test the backoff policy's success path using an asyncio.timeout context."""
    monkeypatch.setattr(mod, "_try_asyncio_timeout", lambda s: AsyncCtx())

    attempts = {"n": 0}

    async def supplier() -> str:
        attempts["n"] += 1
        if attempts["n"] < 3:
            raise RuntimeError("retry me")
        return "OK"

    pol = ExponentialBackoffRetryPolicy(tel)
    out = await pol.run(supplier, 1.0, 3, 0.5, 2.0, 0.0, (RuntimeError,))
    assert out == "OK"
    assert patch_sleep == [0.5, 1.0]
    tel.event.assert_called_with("retry_async_success", {"retries": 2})


@pytest.mark.asyncio
async def test_backoff_policy_failure_with_ctx(
    monkeypatch: pytest.MonkeyPatch, tel: MagicMock
) -> None:
    """Test the backoff policy's failure path after exhausting retries."""
    monkeypatch.setattr(mod, "_try_asyncio_timeout", lambda s: AsyncCtx())

    async def supplier() -> None:
        raise RuntimeError("always")

    pol = ExponentialBackoffRetryPolicy(tel)
    with pytest.raises(RuntimeError, match="always"):
        await pol.run(supplier, 0.5, 2, 0.1, 2.0, 0.0, (RuntimeError,))
    tel.event.assert_any_call("retry_async_failed", {"retries": 2, "error": "always"})


@pytest.mark.asyncio
async def test_backoff_policy_success_fallback_wait_for(
    monkeypatch: pytest.MonkeyPatch, tel: MagicMock
) -> None:
    """Test the backoff policy's success path using the asyncio.wait_for fallback."""
    monkeypatch.setattr(mod, "_try_asyncio_timeout", lambda s: None)

    async def supplier() -> str:
        return "value"

    async def fake_wait_for(coro: Coroutine[Any, Any, Any], timeout: float) -> Any:
        return await coro

    monkeypatch.setattr(asyncio, "wait_for", fake_wait_for)

    pol = ExponentialBackoffRetryPolicy(tel)
    out = await pol.run(supplier, 1.1, 1, 0.2, 2.0, 0.0, (Exception,))
    assert out == "value"
    tel.event.assert_called_with("retry_async_success", {"retries": 0})


def test_backoff_policy_reset_emits(tel: MagicMock) -> None:
    """Test that the backoff policy's reset method emits a telemetry event."""
    ExponentialBackoffRetryPolicy(tel).reset()
    tel.event.assert_called_with("retry_reset", {})


def test_try_asyncio_timeout_no_aenter_aexit(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an object without context manager methods is not used as a timeout context."""

    class Dummy:
        pass

    monkeypatch.setattr("asyncio.timeout", lambda seconds: Dummy())
    Dummy.__module__ = "asyncio"
    assert _try_asyncio_timeout(1) is None


@pytest.mark.asyncio
async def test_backoff_loop_with_jitter() -> None:
    """Test that the backoff loop correctly applies jitter to sleep delays."""
    calls: list[int] = []

    async def supplier() -> str:
        if not calls:
            calls.append(1)
            raise ValueError("fail")
        return "ok"

    tel = MagicMock()
    result = await _backoff_loop(
        supplier,
        retries=2,
        delay=0.01,
        backoff=1,
        jitter=0.5,  # Ensure branch
        retry_on=(ValueError,),
        telemetry=tel,
    )
    assert result == "ok"
    tel.event.assert_any_call("retry_async_success", {"retries": 1})


@pytest.mark.asyncio
async def test_backoff_loop_failure_event() -> None:
    """Test that a failure event is emitted when the backoff loop is exhausted."""

    async def supplier() -> None:
        raise ValueError("fail")

    tel = MagicMock()
    with pytest.raises(ValueError, match="fail"):
        await _backoff_loop(
            supplier,
            retries=2,
            delay=0.01,
            backoff=1,
            jitter=0.5,
            retry_on=(ValueError,),
            telemetry=tel,
        )
    tel.event.assert_any_call("retry_async_failed", {"retries": 2, "error": "fail"})


@pytest.mark.asyncio
async def test_backoff_policy_wait_for_timeouts_then_success(
    monkeypatch: pytest.MonkeyPatch, tel: MagicMock, patch_sleep: list[float]
) -> None:
    """Test the wait_for fallback path where initial attempts time out."""
    monkeypatch.setattr(mod, "_try_asyncio_timeout", lambda s: None)

    attempts = {"n": 0}

    async def supplier() -> str:
        return "OK"

    async def wait_for_with_timeouts(
        coro: Coroutine[Any, Any, Any], timeout: float
    ) -> Any:
        attempts["n"] += 1
        if attempts["n"] < 3:
            with suppress(Exception):
                await coro
            raise TimeoutError("wf timeout")
        return await coro

    monkeypatch.setattr(asyncio, "wait_for", wait_for_with_timeouts)

    pol = ExponentialBackoffRetryPolicy(tel)
    out = await pol.run(
        supplier, seconds=0.4, retries=3, delay=0.25, backoff=2.0, jitter=0.0
    )
    assert out == "OK"
    assert patch_sleep == [0.25, 0.5]
    tel.event.assert_called_with("retry_async_success", {"retries": 2})
