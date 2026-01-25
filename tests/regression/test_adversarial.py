# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Adversarial regression tests for CLI edge cases."""

from __future__ import annotations

import pytest

from tests.regression.test_functional import cli


@pytest.mark.parametrize(
    "tokens",
    [
        ("config", "set", "bad key=1"),
        ("config", "set", "=value"),
        ("plugins", "install", "bad name"),
    ],
    ids=lambda t: " ".join(t),
)
def test_invalid_inputs_fail_loudly(tokens: tuple[str, ...]) -> None:
    r = cli(*tokens, json_output=True, expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    if isinstance(data, dict):
        msg = str(data.get("error", ""))
        failure = str(data.get("failure", ""))
        assert msg or failure
    else:
        assert r.stderr.strip() or r.stdout.strip()


@pytest.mark.parametrize(
    "tokens",
    [
        ("config", "list", "--log-level", "nope"),
        ("config", "get", "missing", "--format", "nope"),
    ],
    ids=lambda t: " ".join(t),
)
def test_conflicting_or_invalid_flags(tokens: tuple[str, ...]) -> None:
    r = cli(*tokens, json_output=True, expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    if isinstance(data, dict):
        msg = str(data.get("error", ""))
        failure = str(data.get("failure", ""))
        assert msg or failure


def test_sleep_rejects_negative_duration() -> None:
    r = cli("sleep", "-1", json_output=True, expect_exit_code=None)
    assert r.returncode in (1, 2)
    data = r.json_err or r.json_out
    if isinstance(data, dict):
        msg = str(data.get("error", ""))
        assert msg
