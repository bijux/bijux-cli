# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly fuzzed precedence combinations."""

from __future__ import annotations

import json
import re

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import assert_config_consistent, assert_no_traceback

pytestmark = [pytest.mark.e2e, pytest.mark.nightly]


def _extract_json_payload(text: str) -> str:
    ansi = re.compile(r"\x1b\[[0-9;]*m")
    for line in reversed(text.splitlines()):
        cleaned = ansi.sub("", line).strip()
        if cleaned.startswith("{") and cleaned.endswith("}"):
            return cleaned
    return ""


def _json_value(payload: str) -> str:
    data = json.loads(payload)
    assert isinstance(data, dict)
    return str(data["value"])


@settings(max_examples=25, deadline=None)
@given(
    key=st.sampled_from(["prec_x", "prec_y", "prec_z"]),
    cfg_val=st.sampled_from(["cfg", "cfg2"]),
    env_val=st.sampled_from(["env", "env2"]),
    cli_val=st.sampled_from(["cli", "cli2"]),
    use_env=st.booleans(),
    use_cli=st.booleans(),
    use_json=st.booleans(),
)
def test_precedence_combinations(
    key: str,
    cfg_val: str,
    env_val: str,
    cli_val: str,
    use_env: bool,
    use_cli: bool,
    use_json: bool,
) -> None:
    with E2EHarness() as h:
        res_set = h.run(["config", "set", f"{key}={cfg_val}"])
        assert res_set.returncode == 0
        assert_no_traceback(res_set.stdout + res_set.stderr)

        if use_cli:
            res_cli = h.run(["config", "set", f"{key}={cli_val}"])
            assert res_cli.returncode == 0
            assert_no_traceback(res_cli.stdout + res_cli.stderr)

        extra_env: dict[str, str] = {}
        if use_env:
            extra_env[f"BIJUXCLI_{key.upper()}"] = env_val

        get_args = ["config", "get", key]
        if use_json:
            get_args += ["--format", "json"]
        res = h.run(get_args, extra_env=extra_env)
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        payload = _extract_json_payload(res.stdout + res.stderr) if use_json else ""
        expected = env_val if use_env else (cli_val if use_cli else cfg_val)
        if use_json:
            if not payload:
                payload = res.stdout.strip()
            if payload.startswith("{"):
                assert _json_value(payload) == expected
            else:
                assert expected in res.stdout
        else:
            assert expected in res.stdout

        assert_config_consistent(h)
