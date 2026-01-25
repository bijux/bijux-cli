# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Intent: compositional command sequences with persistent state.

Why E2E: ordering effects and restarts only show up at the CLI boundary.
Primary invariants: config consistency, exit code stability, no corruption.
"""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import (
    assert_config_consistent,
    assert_exit_code_stable,
    assert_no_state_corruption,
    assert_no_traceback,
    capture_state,
)
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow, pytest.mark.compositional]


def _assert_config_contains(h: E2EHarness, key: str, value: str) -> None:
    content = h.config_path.read_text(encoding="utf-8")
    assert f"BIJUXCLI_{key.upper()}={value}" in content


def _restart(h: E2EHarness) -> E2EHarness:
    return E2EHarness(root=h.root)


@pytest.mark.parametrize(
    "args",
    [
        ["config", "get", "missing"],
        ["config", "unset", "missing"],
        ["plugins", "info", "ghost"],
        ["plugins", "uninstall", "ghost"],
        ["plugins", "check", "ghost"],
        ["config", "export"],
        ["config", "load"],
        ["plugins", "install", "--force"],
        ["config", "set"],
        ["config", "set", "badpair"],
        ["config", "set", "foo=bar", "--format", "nope"],
        ["config", "set", "foo=bar", "--log-level", "nope"],
        ["config", "export", "/no/such/path/out.env"],
        ["config", "load", "/no/such/path/in.env"],
        ["config", "load", "/no/such/path/other.env"],
        ["plugins", "install", "/no/such/path"],
        ["plugins", "install", "bad name"],
        ["plugins", "install", "bad@name"],
        ["plugins", "install", "/no/such/path/again"],
        ["plugins", "info"],
        ["plugins", "uninstall"],
        ["plugins", "check"],
        ["config", "get"],
    ],
)
@pytest.mark.core
def test_invalid_ordering_does_not_corrupt_state(args: list[str]) -> None:
    with E2EHarness() as h:
        assert h.run(["config", "set", "alpha=1"]).returncode == 0
        _assert_config_contains(h, "alpha", "1")
        before = capture_state(h)

        h2 = _restart(h)
        res = h2.run(args)
        assert res.returncode != 0
        assert_no_traceback(res.stdout + res.stderr)

        h3 = _restart(h)
        res_check = h3.run(["config", "get", "alpha"])
        assert res_check.returncode == 0
        assert_no_traceback(res_check.stdout + res_check.stderr)
        _assert_config_contains(h3, "alpha", "1")
        assert_no_state_corruption(before, capture_state(h3))
        assert_config_consistent(h3)


@pytest.mark.parametrize(
    ("sequence", "expect_codes"),
    [
        (
            [
                ["config", "set", "foo=1"],
                ["config", "get", "foo"],
                ["config", "unset", "foo"],
            ],
            [0, 0, 0],
        ),
        (
            [
                ["config", "set", "foo=1"],
                ["config", "unset", "foo"],
                ["config", "get", "foo"],
            ],
            [0, 0, 2],
        ),
    ],
)
def test_valid_ordering_sequences(
    sequence: list[list[str]], expect_codes: list[int]
) -> None:
    with E2EHarness() as h:
        res1 = h.run(sequence[0])
        assert res1.returncode == expect_codes[0]
        assert_no_traceback(res1.stdout + res1.stderr)

        h2 = _restart(h)
        res2 = h2.run(sequence[1])
        assert res2.returncode == expect_codes[1]
        assert_no_traceback(res2.stdout + res2.stderr)

        h3 = _restart(h2)
        res3 = h3.run(sequence[2])
        assert res3.returncode == expect_codes[2]
        assert_no_traceback(res3.stdout + res3.stderr)
        if expect_codes[1] == expect_codes[2]:
            assert_exit_code_stable([res2.returncode, res3.returncode])
        assert_config_consistent(h3)


@pytest.mark.parametrize(
    "name",
    [
        "order_plugin_a",
        "order_plugin_b",
        "order_plugin_c",
        "order_plugin_d",
        "order_plugin_e",
    ],
)
@pytest.mark.core
def test_plugin_ordering_list_info_uninstall_then_install(name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        res_list = h.run(["plugins", "list"])
        assert res_list.returncode == 0
        assert_no_traceback(res_list.stdout + res_list.stderr)

        h2 = _restart(h)
        res_info = h2.run(["plugins", "info", name])
        assert res_info.returncode != 0
        assert_no_traceback(res_info.stdout + res_info.stderr)

        h3 = _restart(h2)
        res_uninstall = h3.run(["plugins", "uninstall", name])
        assert res_uninstall.returncode != 0
        assert_no_traceback(res_uninstall.stdout + res_uninstall.stderr)

        h4 = _restart(h3)
        res_install = h4.run(["plugins", "install", str(dummy_dir)])
        assert res_install.returncode == 0
        assert_no_traceback(res_install.stdout + res_install.stderr)
        assert (h4.plugins_dir / name).exists()

        h5 = _restart(h4)
        res_info2 = h5.run(["plugins", "info", name])
        assert res_info2.returncode == 0
        assert_no_traceback(res_info2.stdout + res_info2.stderr)
        assert_config_consistent(h5)
