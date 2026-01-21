# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Retry adapters for transient failures."""

from __future__ import annotations

import random
import time
from collections.abc import Callable
from typing import Any, TypeVar

T = TypeVar("T")


class NoopRetryPolicy:
    """No-op retry policy that calls the function once."""

    def run(self, func: Callable[[], T]) -> T:
        return func()


class TimeoutRetryPolicy:
    """Retries a callable until a timeout is reached."""

    def __init__(self, telemetry: Any, timeout: float = 5.0) -> None:
        self._telemetry = telemetry
        self._timeout = timeout

    def run(self, func: Callable[[], T]) -> T:
        start = time.time()
        last_error: Exception | None = None
        while time.time() - start < self._timeout:
            try:
                return func()
            except Exception as exc:
                last_error = exc
                self._telemetry.event("retry_attempt_failed", {"error": str(exc)})
                time.sleep(0.05)
        raise RuntimeError(f"Retry timeout after {self._timeout}s: {last_error}")


class ExponentialBackoffRetryPolicy:
    """Retries with exponential backoff and jitter."""

    def __init__(
        self,
        telemetry: Any,
        *,
        base_delay: float = 0.1,
        max_delay: float = 2.0,
        max_attempts: int = 5,
    ) -> None:
        self._telemetry = telemetry
        self._base_delay = base_delay
        self._max_delay = max_delay
        self._max_attempts = max_attempts

    def run(self, func: Callable[[], T]) -> T:
        attempt = 0
        last_error: Exception | None = None
        while attempt < self._max_attempts:
            try:
                return func()
            except Exception as exc:
                last_error = exc
                attempt += 1
                delay = min(self._base_delay * (2**attempt), self._max_delay)
                delay = delay + random.uniform(0.0, delay / 4)
                self._telemetry.event(
                    "retry_backoff",
                    {"attempt": attempt, "delay": delay, "error": str(exc)},
                )
                time.sleep(delay)
        raise RuntimeError(f"Retry attempts exhausted: {last_error}")


__all__ = ["NoopRetryPolicy", "TimeoutRetryPolicy", "ExponentialBackoffRetryPolicy"]
