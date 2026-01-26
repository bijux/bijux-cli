# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly fuzzed CLI command sequences."""

from __future__ import annotations

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import (
    assert_config_consistent,
    assert_no_traceback,
    assert_plugins_consistent,
)
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.nightly]


Action = tuple[str, str]


def _action_strategy() -> st.SearchStrategy[Action]:
    return st.one_of(
        st.tuples(st.just("config_set"), st.sampled_from(["alpha", "beta", "gamma"])),
        st.tuples(st.just("config_unset"), st.sampled_from(["alpha", "beta", "gamma"])),
        st.tuples(st.just("config_get"), st.sampled_from(["alpha", "beta", "gamma"])),
        st.tuples(st.just("plugin_install"), st.sampled_from(["p1", "p2", "p3"])),
        st.tuples(st.just("plugin_uninstall"), st.sampled_from(["p1", "p2", "p3"])),
        st.tuples(st.just("history_clear"), st.just("history")),
    )


@settings(max_examples=5, deadline=None)
@given(st.lists(_action_strategy(), min_size=50, max_size=200))
def test_fuzzed_cli_sequences(actions: list[Action]) -> None:
    with E2EHarness() as h:
        for action, value in actions:
            if action == "config_set":
                res = h.run(["config", "set", f"{value}=1"])
            elif action == "config_unset":
                res = h.run(["config", "unset", value])
            elif action == "config_get":
                res = h.run(["config", "get", value])
            elif action == "plugin_install":
                plugin_dir = write_dummy_plugin(h.root / value, name=value)
                res = h.run(["plugins", "install", str(plugin_dir)])
            elif action == "plugin_uninstall":
                res = h.run(["plugins", "uninstall", value])
            elif action == "history_clear":
                res = h.run(["history", "clear"])
            else:
                raise AssertionError(f"unknown action: {action}")

            assert res.returncode in (0, 1, 2)
            assert_no_traceback(res.stdout + res.stderr)
            assert_config_consistent(h)
            assert_plugins_consistent(h)

        assert_config_consistent(h)
        assert_plugins_consistent(h)
