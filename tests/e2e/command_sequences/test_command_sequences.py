# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Command sequence E2E tests."""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _no_traceback(text: str) -> None:
    assert "traceback" not in text.lower()


def _assert_config_contains(h: E2EHarness, key: str, value: str) -> None:
    content = h.config_path.read_text(encoding="utf-8")
    assert f"BIJUXCLI_{key.upper()}={value}" in content


@pytest.mark.parametrize(
    "args",
    [
        ["config", "get", "missing"],
        ["config", "unset", "missing"],
        ["plugins", "info", "ghost"],
        ["plugins", "uninstall", "ghost"],
        ["plugins", "check", "ghost"],
        ["config", "set"],
        ["config", "set", "badpair"],
        ["config", "export", "/no/such/path/out.env"],
        ["config", "load", "/no/such/path/in.env"],
        ["plugins", "install", "/no/such/path"],
        ["plugins", "install", "bad name"],
        ["plugins", "info"],
        ["plugins", "uninstall"],
        ["plugins", "check"],
        ["config", "get"],
    ],
)
def test_invalid_ordering_does_not_corrupt_state(args: list[str]) -> None:
    with E2EHarness() as h:
        assert h.run(["config", "set", "alpha=1"]).returncode == 0
        _assert_config_contains(h, "alpha", "1")

        res = h.run(args)
        assert res.returncode != 0
        _no_traceback(res.stdout + res.stderr)

        res_check = h.run(["config", "get", "alpha"])
        assert res_check.returncode == 0
        _no_traceback(res_check.stdout + res_check.stderr)
        _assert_config_contains(h, "alpha", "1")


@pytest.mark.parametrize(
    ("sequence", "expect_codes"),
    [
        ([["plugins", "list"], ["plugins", "list"]], [0, 0]),
        ([["config", "set", "foo=1"], ["config", "get", "foo"]], [0, 0]),
        ([["config", "set", "foo=1"], ["config", "unset", "foo"]], [0, 0]),
        (
            [
                ["config", "set", "foo=1"],
                ["config", "unset", "foo"],
                ["config", "get", "foo"],
            ],
            [0, 0, 2],
        ),
        (
            [["config", "set", "alpha=1"], ["config", "set", "beta=2"]],
            [0, 0],
        ),
    ],
)
def test_valid_ordering_sequences(
    sequence: list[list[str]], expect_codes: list[int]
) -> None:
    with E2EHarness() as h:
        for args, expected in zip(sequence, expect_codes, strict=True):
            res = h.run(args)
            assert res.returncode == expected
            _no_traceback(res.stdout + res.stderr)
        saw_set_foo = any(args[:3] == ["config", "set", "foo=1"] for args in sequence)
        saw_unset_foo = any(args[:3] == ["config", "unset", "foo"] for args in sequence)
        if saw_set_foo and not saw_unset_foo:
            _assert_config_contains(h, "foo", "1")


@pytest.mark.parametrize(
    "name",
    [
        "order_plugin_a",
        "order_plugin_b",
        "order_plugin_c",
        "order_plugin_d",
        "order_plugin_e",
        "order_plugin_f",
        "order_plugin_g",
        "order_plugin_h",
        "order_plugin_i",
        "order_plugin_j",
    ],
)
def test_plugin_ordering_list_info_uninstall_then_install(name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        res_list = h.run(["plugins", "list"])
        _no_traceback(res_list.stdout + res_list.stderr)
        assert res_list.returncode == 0

        res_info = h.run(["plugins", "info", name])
        assert res_info.returncode != 0
        _no_traceback(res_info.stdout + res_info.stderr)

        res_uninstall = h.run(["plugins", "uninstall", name])
        assert res_uninstall.returncode != 0
        _no_traceback(res_uninstall.stdout + res_uninstall.stderr)

        res_install = h.run(["plugins", "install", str(dummy_dir)])
        assert res_install.returncode == 0
        _no_traceback(res_install.stdout + res_install.stderr)
        assert (h.plugins_dir / name).exists()

        res_info2 = h.run(["plugins", "info", name])
        assert res_info2.returncode == 0
        _no_traceback(res_info2.stdout + res_info2.stderr)
