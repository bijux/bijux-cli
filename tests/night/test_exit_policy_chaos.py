# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly exit policy chaos tests."""

from __future__ import annotations

import json
import re

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import (
    assert_config_consistent,
    assert_exit_code_stable,
    assert_no_state_corruption,
    assert_no_traceback,
    capture_state,
)

pytestmark = [pytest.mark.e2e, pytest.mark.night]


def _extract_json_payload(text: str) -> str:
    ansi = re.compile(r"\x1b\[[0-9;]*m")
    for line in reversed(text.splitlines()):
        cleaned = ansi.sub("", line).strip()
        if cleaned.startswith("{") and cleaned.endswith("}"):
            return cleaned
    return ""


def _json_error(payload: str) -> str:
    data = json.loads(payload)
    assert isinstance(data, dict)
    return str(data.get("error", ""))


FAILURE_ARGS = [
    ["config", "set", "badpair"],
    ["config", "get", "missing_key"],
    ["config", "unset", "missing_key"],
    ["config", "set", "foo=bar", "--format", "nope"],
    ["plugins", "info", "missing_plugin"],
    ["plugins", "uninstall", "missing_plugin"],
    ["does-not-exist"],
]


@settings(max_examples=25, deadline=None)
@given(
    failure=st.sampled_from(FAILURE_ARGS),
    use_quiet=st.booleans(),
    use_json=st.booleans(),
    log_level=st.sampled_from(["trace", "debug", "info"]),
)
def test_exit_policy_chaos(
    failure: list[str], use_quiet: bool, use_json: bool, log_level: str
) -> None:
    with E2EHarness() as h:
        ok = h.run(["config", "set", "guard=1"])
        assert ok.returncode == 0
        before = capture_state(h)

        args = list(failure)
        if use_quiet:
            args.append("--quiet")
        if use_json:
            args += ["--format", "json"]
        args += ["--log-level", log_level]

        res = h.run(args)
        assert res.returncode in (1, 2)
        if not use_json and not use_quiet and log_level != "trace":
            assert_no_traceback(res.stdout + res.stderr)
        assert_no_state_corruption(before, capture_state(h))
        assert_exit_code_stable([res.returncode, res.returncode])

        if use_quiet:
            assert res.stdout.strip() == ""
        if use_json:
            payload = _extract_json_payload(res.stdout + res.stderr)
            if payload:
                assert _json_error(payload) != ""

        assert_config_consistent(h)
