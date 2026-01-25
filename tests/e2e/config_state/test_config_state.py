# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Intent: stateful config mutations and reversibility.

Why E2E: only CLI sequences reveal persistence and idempotence issues.
Primary invariants: config consistency, no corruption, no traceback.
"""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import (
    assert_config_consistent,
    assert_no_state_corruption,
    assert_no_traceback,
    capture_state,
)

pytestmark = [pytest.mark.e2e, pytest.mark.slow, pytest.mark.stateful]


def _assert_config_contains(h: E2EHarness, key: str, value: str) -> None:
    content = h.config_path.read_text(encoding="utf-8")
    assert f"BIJUXCLI_{key.upper()}={value}" in content


def _restart(h: E2EHarness) -> E2EHarness:
    return E2EHarness(root=h.root)


@pytest.mark.parametrize(
    ("key", "value"),
    [
        ("alpha", "1"),
        ("beta", "2"),
        ("gamma", "3"),
        ("delta", "4"),
        ("epsilon", "5"),
        ("zeta", "6"),
        ("eta", "7"),
        ("theta", "8"),
        ("iota", "9"),
        ("kappa", "10"),
    ],
)
@pytest.mark.core
def test_config_set_unset_set_again(key: str, value: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}={value}"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        assert h.config_path.exists()
        _assert_config_contains(h, key, value)
        assert_config_consistent(h)

        h2 = _restart(h)
        res = h2.run(["config", "unset", key])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        assert f"BIJUXCLI_{key.upper()}=" not in h2.config_path.read_text(
            encoding="utf-8"
        )
        assert_config_consistent(h2)

        h3 = _restart(h2)
        res = h3.run(["config", "set", f"{key}={value}"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        _assert_config_contains(h3, key, value)
        assert_config_consistent(h3)


@pytest.mark.parametrize(
    ("primary", "secondary"),
    [
        ("foo", "bar"),
        ("one", "two"),
        ("left", "right"),
        ("hot", "cold"),
        ("red", "blue"),
        ("up", "down"),
    ],
)
def test_config_unset_preserves_other_keys(primary: str, secondary: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{primary}=1"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h2 = _restart(h)
        res = h2.run(["config", "set", f"{secondary}=2"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h3 = _restart(h2)
        res = h3.run(["config", "unset", primary])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        content = h3.config_path.read_text(encoding="utf-8")
        assert f"BIJUXCLI_{primary.upper()}=" not in content
        assert f"BIJUXCLI_{secondary.upper()}=2" in content
        assert_config_consistent(h3)


@pytest.mark.parametrize(
    ("key", "value"),
    [
        ("idemp_a", "1"),
        ("idemp_b", "2"),
        ("idemp_c", "3"),
        ("idemp_d", "4"),
        ("idemp_e", "5"),
    ],
)
@pytest.mark.core
def test_config_set_is_idempotent(key: str, value: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}={value}"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        before = capture_state(h)

        h2 = _restart(h)
        res = h2.run(["config", "set", f"{key}={value}"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h3 = _restart(h2)
        res = h3.run(["config", "set", f"{key}={value}"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        assert_config_consistent(h3)
        assert_no_state_corruption(before, capture_state(h3))


@pytest.mark.parametrize(
    "key",
    [
        "unset_a",
        "unset_b",
        "unset_c",
        "unset_d",
        "unset_e",
    ],
)
@pytest.mark.core
def test_config_unset_is_idempotent(key: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}=1"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h2 = _restart(h)
        res = h2.run(["config", "unset", key])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h3 = _restart(h2)
        res = h3.run(["config", "unset", key])
        assert res.returncode == 1
        assert_no_traceback(res.stdout + res.stderr)
        content = h3.config_path.read_text(encoding="utf-8")
        assert f"BIJUXCLI_{key.upper()}=" not in content
        assert_config_consistent(h3)


@pytest.mark.parametrize(
    "key",
    [
        "repeat_a",
        "repeat_b",
    ],
)
def test_config_get_repeat_is_stable(key: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}=1"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        before = capture_state(h)

        h2 = _restart(h)
        res = h2.run(["config", "get", key])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h3 = _restart(h2)
        res = h3.run(["config", "get", key])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        assert_no_state_corruption(before, capture_state(h3))
        assert_config_consistent(h3)
