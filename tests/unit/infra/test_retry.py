# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra retry module."""

from __future__ import annotations

from unittest.mock import MagicMock

import pytest

from bijux_cli.infra.retry import (
    ExponentialBackoffRetryPolicy,
    NoopRetryPolicy,
    TimeoutRetryPolicy,
)
from bijux_cli.infra.telemetry import Telemetry


def test_noop_retry_policy_calls_once() -> None:
    """NoopRetryPolicy should invoke the function exactly once."""
    policy = NoopRetryPolicy()
    called = {"n": 0}

    def work() -> int:
        called["n"] += 1
        return 7

    assert policy.run(work) == 7
    assert called["n"] == 1


def test_timeout_retry_policy_raises() -> None:
    """TimeoutRetryPolicy should raise after timeout expires."""
    tel = MagicMock(spec=Telemetry)
    policy = TimeoutRetryPolicy(tel, timeout=0.01)

    def fail() -> None:
        raise RuntimeError("boom")

    with pytest.raises(RuntimeError, match="Retry timeout"):
        policy.run(fail)


def test_exponential_backoff_succeeds() -> None:
    """ExponentialBackoffRetryPolicy should return once a call succeeds."""
    tel = MagicMock(spec=Telemetry)
    policy = ExponentialBackoffRetryPolicy(tel, max_attempts=3, base_delay=0.0)
    called = {"n": 0}

    def work() -> str:
        called["n"] += 1
        if called["n"] < 2:
            raise RuntimeError("fail")
        return "ok"

    assert policy.run(work) == "ok"
    assert called["n"] == 2
