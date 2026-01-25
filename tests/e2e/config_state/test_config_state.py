# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Config state E2E tests."""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _no_traceback(text: str) -> None:
    assert "traceback" not in text.lower()


def _assert_config_contains(h: E2EHarness, key: str, value: str) -> None:
    content = h.config_path.read_text(encoding="utf-8")
    assert f"BIJUXCLI_{key.upper()}={value}" in content


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
def test_config_set_unset_set_again(key: str, value: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}={value}"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        assert h.config_path.exists()
        _assert_config_contains(h, key, value)

        res = h.run(["config", "unset", key])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        content = h.config_path.read_text(encoding="utf-8")
        assert f"BIJUXCLI_{key.upper()}=" not in content

        res = h.run(["config", "set", f"{key}={value}"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        _assert_config_contains(h, key, value)


@pytest.mark.parametrize(
    ("primary", "secondary"),
    [
        ("foo", "bar"),
        ("one", "two"),
        ("left", "right"),
        ("hot", "cold"),
        ("red", "blue"),
    ],
)
def test_config_unset_preserves_other_keys(primary: str, secondary: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{primary}=1"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        res = h.run(["config", "set", f"{secondary}=2"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        res = h.run(["config", "unset", primary])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        content = h.config_path.read_text(encoding="utf-8")
        assert f"BIJUXCLI_{primary.upper()}=" not in content
        assert f"BIJUXCLI_{secondary.upper()}=2" in content


@pytest.mark.parametrize(
    ("key", "value"),
    [
        ("idemp_a", "1"),
        ("idemp_b", "2"),
        ("idemp_c", "3"),
        ("idemp_d", "4"),
        ("idemp_e", "5"),
        ("idemp_f", "6"),
        ("idemp_g", "7"),
        ("idemp_h", "8"),
        ("idemp_i", "9"),
        ("idemp_j", "10"),
    ],
)
def test_config_set_is_idempotent(key: str, value: str) -> None:
    with E2EHarness() as h:
        for _ in range(3):
            res = h.run(["config", "set", f"{key}={value}"])
            assert res.returncode == 0
            _no_traceback(res.stdout + res.stderr)
        content = h.config_path.read_text(encoding="utf-8")
        assert content.count(f"BIJUXCLI_{key.upper()}=") == 1


@pytest.mark.parametrize(
    "key",
    [
        "unset_a",
        "unset_b",
        "unset_c",
        "unset_d",
        "unset_e",
        "unset_f",
        "unset_g",
        "unset_h",
        "unset_i",
        "unset_j",
    ],
)
def test_config_unset_is_idempotent(key: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}=1"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        res = h.run(["config", "unset", key])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        for _ in range(2):
            res = h.run(["config", "unset", key])
            assert res.returncode == 1
            _no_traceback(res.stdout + res.stderr)
        content = h.config_path.read_text(encoding="utf-8")
        assert f"BIJUXCLI_{key.upper()}=" not in content


@pytest.mark.parametrize(
    "key",
    [
        "repeat_a",
        "repeat_b",
        "repeat_c",
        "repeat_d",
        "repeat_e",
    ],
)
def test_config_get_repeat_is_stable(key: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}=1"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        before = h.config_path.read_text(encoding="utf-8")
        for _ in range(5):
            res = h.run(["config", "get", key])
            assert res.returncode == 0
            _no_traceback(res.stdout + res.stderr)
        after = h.config_path.read_text(encoding="utf-8")
        assert before == after
