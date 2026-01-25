# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly API/CLI parity fuzz tests."""

from __future__ import annotations

from hypothesis import HealthCheck, given, settings
from hypothesis import strategies as st
import pytest

from bijux_cli.api.facade import BijuxAPI
from bijux_cli.core.di import DIContainer
from bijux_cli.services.config.contracts import ConfigProtocol
from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import assert_config_consistent, assert_no_traceback

pytestmark = [pytest.mark.e2e, pytest.mark.night]


def _sync_api_env(monkeypatch: pytest.MonkeyPatch, h: E2EHarness) -> None:
    for key, value in h.env.items():
        monkeypatch.setenv(key, value)


def _register_config_helpers(api: BijuxAPI) -> None:
    def _cfg_set(key: str, value: str) -> None:
        DIContainer.current().resolve(ConfigProtocol).set(key, value)

    def _cfg_get(key: str) -> str:
        return str(DIContainer.current().resolve(ConfigProtocol).get(key, default=""))

    api.register("cfg_set", _cfg_set)
    api.register("cfg_get", _cfg_get)


Action = tuple[str, str, str]


def _action_strategy() -> st.SearchStrategy[Action]:
    key = st.sampled_from(["par_a", "par_b", "par_c"])
    value = st.sampled_from(["1", "2", "3"])
    return st.one_of(
        st.tuples(st.just("cli_set"), key, value),
        st.tuples(st.just("api_set"), key, value),
        st.tuples(st.just("cli_get"), key, value),
        st.tuples(st.just("api_get"), key, value),
    )


@settings(
    max_examples=10,
    deadline=None,
    suppress_health_check=[HealthCheck.function_scoped_fixture],
)
@given(st.lists(_action_strategy(), min_size=30, max_size=80))
def test_api_cli_parity_fuzz(
    monkeypatch: pytest.MonkeyPatch, actions: list[Action]
) -> None:
    with E2EHarness() as h:
        _sync_api_env(monkeypatch, h)
        api = BijuxAPI()
        _register_config_helpers(api)

        for action, key, value in actions:
            if action == "cli_set":
                res = h.run(["config", "set", f"{key}={value}"])
                assert res.returncode == 0
                assert_no_traceback(res.stdout + res.stderr)
            elif action == "api_set":
                api.run_sync("cfg_set", key, value)
            elif action == "cli_get":
                res = h.run(["config", "get", key])
                assert res.returncode in (0, 1, 2)
                assert_no_traceback(res.stdout + res.stderr)
            elif action == "api_get":
                api.run_sync("cfg_get", key)
            else:
                raise AssertionError(f"unknown action: {action}")

        assert_config_consistent(h)
