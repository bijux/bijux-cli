# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for operation retry policies.

This module specifies the `RetryPolicyProtocol`, a formal interface that any
service providing retry logic for potentially failing operations must
implement. This is particularly useful for handling transient errors in
network requests or other I/O-bound tasks.
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from typing import Protocol, TypeVar, runtime_checkable

T = TypeVar("T")


@runtime_checkable
class RetryPolicyProtocol(Protocol):
    """Defines the contract for retry policies in asynchronous operations.

    This interface specifies the methods for executing an operation with retry
    logic (e.g., exponential backoff) and for resetting the policy's internal
    state.
    """

    async def run(
        self, supplier: Callable[[], Awaitable[T]], seconds: float = 1.0
    ) -> T:
        """Runs an asynchronous operation with a retry policy.

        Implementations of this method will repeatedly call the `supplier`
        until it succeeds or the retry policy is exhausted.

        Args:
            supplier (Callable[[], Awaitable[T]]): A no-argument function that
                returns an awaitable (e.g., a coroutine).
            seconds (float): The timeout for each attempt in seconds.

        Returns:
            T: The successful result of the operation.
        """
        ...

    def reset(self) -> None:
        """Resets the internal state of the retry policy.

        This is useful for reusing a policy instance for a new, independent
        operation.
        """
        ...
