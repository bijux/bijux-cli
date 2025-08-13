# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides concrete asynchronous retry policy implementations.

This module defines classes that implement the `RetryPolicyProtocol` to
handle transient errors in asynchronous operations. It offers two main
strategies:

    * `TimeoutRetryPolicy`: A simple policy that applies a single timeout to an
        operation.
    * `ExponentialBackoffRetryPolicy`: A more advanced policy that retries an
        operation multiple times with an exponentially increasing delay and
        random jitter between attempts.

These components are designed to be used by services to build resilience
against temporary failures, such as network issues.
"""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from contextlib import AbstractAsyncContextManager, suppress
import inspect
import secrets
from typing import Any, TypeVar, cast

from injector import inject

from bijux_cli.contracts import RetryPolicyProtocol, TelemetryProtocol
from bijux_cli.core.exceptions import BijuxError

T = TypeVar("T")


def _close_awaitable(obj: Any) -> None:
    """Safely closes an object if it has a synchronous `close()` method.

    This helper performs a best-effort call to `obj.close()`, suppressing any
    exceptions that may be raised.

    Args:
        obj (Any): The object to close.

    Returns:
        None:
    """
    close = getattr(obj, "close", None)
    if callable(close):
        with suppress(Exception):
            close()


def _try_asyncio_timeout(
    seconds: float,
) -> AbstractAsyncContextManager[None] | None:
    """Returns an `asyncio.timeout` context manager if it is available and usable.

    This function checks if `asyncio.timeout` is a real, usable implementation,
    avoiding mocks or incompatible objects that may exist in some test
    environments.

    Args:
        seconds (float): The timeout duration in seconds.

    Returns:
        AbstractAsyncContextManager[None] | None: A configured timeout context
            manager if supported, otherwise `None`.
    """
    async_timeout = getattr(asyncio, "timeout", None)

    if (
        async_timeout is None
        or not callable(async_timeout)
        or getattr(async_timeout, "__module__", "") == "unittest.mock"
    ):
        return None

    try:
        candidate = async_timeout(seconds)
    except (TypeError, ValueError, RuntimeError):
        return None

    if inspect.isawaitable(candidate):
        _close_awaitable(candidate)
        return None

    if hasattr(candidate, "__aenter__") and hasattr(candidate, "__aexit__"):
        return cast(AbstractAsyncContextManager[None], candidate)

    return None


async def _backoff_loop(
    supplier: Callable[[], Awaitable[T]],
    *,
    retries: int,
    delay: float,
    backoff: float,
    jitter: float,
    retry_on: tuple[type[BaseException], ...],
    telemetry: TelemetryProtocol,
) -> T:
    """Executes an async operation with an exponential-backoff retry loop.

    Args:
        supplier (Callable[[], Awaitable[T]]): The async function to execute.
        retries (int): The maximum number of retry attempts.
        delay (float): The initial delay in seconds before the first retry.
        backoff (float): The multiplier applied to the delay after each failure.
        jitter (float): The random fractional jitter to apply to each delay.
        retry_on (tuple[type[BaseException], ...]): A tuple of exception types
            that will trigger a retry.
        telemetry (TelemetryProtocol): The service for emitting telemetry events.

    Returns:
        T: The result from a successful call to the `supplier`.

    Raises:
        BaseException: The last exception raised by `supplier` if all retries fail.
        RuntimeError: If the loop finishes without returning or raising, which
            should be unreachable.
    """
    attempts = max(1, retries)
    for attempt in range(attempts):
        try:
            result = await supplier()
            telemetry.event("retry_async_success", {"retries": attempt})
            return result
        except retry_on as exc:
            if attempt + 1 == attempts:
                telemetry.event(
                    "retry_async_failed", {"retries": retries, "error": str(exc)}
                )
                raise
            sleep_for = delay * (backoff**attempt)
            if jitter:
                sleep_for += sleep_for * secrets.SystemRandom().uniform(-jitter, jitter)
            await asyncio.sleep(sleep_for)
    raise RuntimeError("Unreachable code")  # pragma: no cover


class TimeoutRetryPolicy(
    RetryPolicyProtocol  # pyright: ignore[reportUntypedBaseClass]
):
    """A retry policy that applies a single, one-time timeout to an operation.

    Attributes:
        _telemetry (TelemetryProtocol): The service for emitting telemetry events.
    """

    @inject
    def __init__(self, telemetry: TelemetryProtocol) -> None:
        """Initializes the `TimeoutRetryPolicy`.

        Args:
            telemetry (TelemetryProtocol): The service for emitting events.
        """
        self._telemetry = telemetry

    async def run(
        self,
        supplier: Callable[[], Awaitable[T]],
        seconds: float = 1.0,
    ) -> T:
        """Executes an awaitable `supplier` with a single timeout.

        This method uses the modern `asyncio.timeout` context manager if
        available, otherwise it falls back to `asyncio.wait_for`.

        Args:
            supplier (Callable[[], Awaitable[T]]): The async operation to run.
            seconds (float): The timeout duration in seconds. Must be positive.

        Returns:
            T: The result of the `supplier` if it completes in time.

        Raises:
            ValueError: If `seconds` is less than or equal to 0.
            BijuxError: If the operation times out.
        """
        if seconds <= 0:
            raise ValueError("seconds must be > 0")

        ctx = _try_asyncio_timeout(seconds)

        try:
            if ctx is not None:
                async with ctx:
                    result = await supplier()
            else:
                result = await asyncio.wait_for(supplier(), timeout=seconds)

            self._telemetry.event("retry_timeout_success", {"seconds": seconds})
            return result

        except TimeoutError as exc:
            self._telemetry.event(
                "retry_timeout_failed", {"seconds": seconds, "error": str(exc)}
            )
            raise BijuxError(
                f"Operation timed out after {seconds}s", http_status=504
            ) from exc

    def reset(self) -> None:
        """Resets the retry policy state. This is a no-op for this policy."""
        self._telemetry.event("retry_reset", {})


class ExponentialBackoffRetryPolicy(
    RetryPolicyProtocol  # pyright: ignore[reportUntypedBaseClass]
):
    """A retry policy with exponential backoff, jitter, and per-attempt timeouts.

    Attributes:
        _telemetry (TelemetryProtocol): The service for emitting telemetry events.
    """

    @inject
    def __init__(self, telemetry: TelemetryProtocol) -> None:
        """Initializes the `ExponentialBackoffRetryPolicy`.

        Args:
            telemetry (TelemetryProtocol): The service for emitting events.
        """
        self._telemetry = telemetry

    async def run(
        self,
        supplier: Callable[[], Awaitable[T]],
        seconds: float = 1.0,
        retries: int = 3,
        delay: float = 1.0,
        backoff: float = 2.0,
        jitter: float = 0.3,
        retry_on: tuple[type[BaseException], ...] = (Exception,),
    ) -> T:
        """Executes a supplier with a timeout and exponential-backoff retries.

        Args:
            supplier (Callable[[], Awaitable[T]]): The async operation to run.
            seconds (float): The timeout for each attempt in seconds. Must be > 0.
            retries (int): The maximum number of retry attempts.
            delay (float): The initial delay in seconds before the first retry.
            backoff (float): The multiplier for the delay after each failure.
            jitter (float): The random fractional jitter to apply to each delay.
            retry_on (tuple[type[BaseException], ...]): A tuple of exception
                types that will trigger a retry.

        Returns:
            T: The result of the `supplier` if one of the attempts succeeds.

        Raises:
            ValueError: If `seconds` is less than or equal to 0.
            BaseException: The last exception raised by `supplier` if all
                attempts fail.
        """
        if seconds <= 0:
            raise ValueError("seconds must be > 0")

        ctx = _try_asyncio_timeout(seconds)

        if ctx is not None:
            async with ctx:
                return await _backoff_loop(
                    supplier,
                    retries=retries,
                    delay=delay,
                    backoff=backoff,
                    jitter=jitter,
                    retry_on=retry_on,
                    telemetry=self._telemetry,
                )
        else:

            async def timed_supplier() -> T:
                """Wraps the supplier in an `asyncio.wait_for` timeout.

                Returns:
                    T: The result of the `supplier` if it completes in time.
                """
                return await asyncio.wait_for(supplier(), timeout=seconds)

            return await _backoff_loop(
                timed_supplier,
                retries=retries,
                delay=delay,
                backoff=backoff,
                jitter=jitter,
                retry_on=retry_on,
                telemetry=self._telemetry,
            )

    def reset(self) -> None:
        """Resets the retry policy state. This is a no-op for this policy."""
        self._telemetry.event("retry_reset", {})


__all__ = ["TimeoutRetryPolicy", "ExponentialBackoffRetryPolicy"]
